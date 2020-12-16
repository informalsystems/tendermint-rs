const _DOCKER_IMAGE: &str = "informaldev/tendermint:0.34.0";
const _CONTAINER_NAME: &str = "kvstore-test";
const _HOST_RPC_PORT: u16 = 26657;
const _CONTAINER_RPC_PORT: u16 = 26657;

#[cfg(feature = "docker")]
use shiplift::{
    rep::Container, ContainerListOptions, ContainerOptions, Docker, PullOptions, RmContainerOptions,
};

fn main() {
    #[cfg(feature = "docker")]
    async_main();
}

#[tokio::main]
#[cfg(feature = "docker")]
async fn async_main() {
    // Make sure the build script executes each time the feature flag is set.
    // This is important to get a fresh docker container for each test.
    use std::env;
    use std::path::Path;
    println!(
        "cargo:rerun-if-changed={}",
        Path::join(Path::new(&env::var("OUT_DIR").unwrap()), "alwaysrerun.txt")
            .to_str()
            .unwrap()
    );

    // Find or pull docker image
    let docker = Docker::new();
    let mut image = docker_find_image(&docker, &_DOCKER_IMAGE.to_string()).await;
    if image.is_none() {
        docker_pull(&docker).await;
        image = docker_find_image(&docker, &_DOCKER_IMAGE.to_string()).await;
        if image.is_none() {
            panic!("docker image does not exist");
        }
    }
    let image = image.unwrap();

    // Find existing container and delete it
    // Todo: make our tests so we can reuse existing chains instead of creating fresh for each test
    let old_container = docker_find_container(&docker, &_CONTAINER_NAME.to_string()).await;
    if let Some(c) = old_container {
        docker_remove_container(&docker, c.id.as_str()).await;
    }

    // Create fresh container
    let container_id = docker_create_container(&docker, _DOCKER_IMAGE, _CONTAINER_NAME).await;

    // Start container
    docker_start_container(&docker, container_id.as_str()).await;

    // Inform the user
    println!(
        "cargo:warning=using docker image {} {}",
        _DOCKER_IMAGE, image
    );
    println!("cargo:warning=`docker stop {}` stops the container", _CONTAINER_NAME);
}

// Return image ID
#[cfg(feature = "docker")]
async fn docker_find_image(docker: &Docker, docker_image: &String) -> Option<String> {
    // Docker Engine API 1.41 removed the "filter" (singular) query parameter in favor of "filters".
    // "filters" can search for images by name using "reference=..." which is not implemented in shiplift 0.6.
    // https://docs-stage.docker.com/engine/api/version-history/
    let images = docker.images().list(&Default::default()).await.unwrap();
    for image in images {
        match image.repo_tags {
            None => continue,
            Some(tags) => {
                if tags.contains(docker_image) {
                    return Some(image.id);
                }
            }
        }
    }
    None
}

#[cfg(feature = "docker")]
async fn docker_pull(docker: &Docker) {
    let mut stream = docker
        .images()
        .pull(&PullOptions::builder().image(_DOCKER_IMAGE).build());

    // Todo: is there a less CPU-intensive way to wait for the stream to finish?
    use futures::StreamExt;
    while let Some(pull_result) = stream.next().await {
        match pull_result {
            Ok(_) => continue,
            Err(e) => eprintln!("cargo:warning=Error: {}", e),
        }
    }
}

// Return container
#[cfg(feature = "docker")]
async fn docker_find_container(docker: &Docker, container_name: &String) -> Option<Container> {
    // The shiplift 0.6.0 ContainerFilter does not support filtering by container name.
    let containers = docker
        .containers()
        .list(&ContainerListOptions::builder().all().build())
        .await
        .unwrap();
    for container in containers {
        if container.names.contains(&format!("/{}", container_name)) {
            return Some(container);
        }
    }
    None
}

// Return container ID
#[cfg(feature = "docker")]
async fn docker_create_container(
    docker: &Docker,
    docker_image: &str,
    container_name: &str,
) -> String {
    let docker_container = docker
        .containers()
        .create(
            &ContainerOptions::builder(docker_image)
                .name(format!("/{}", container_name).as_str())
                .expose(_CONTAINER_RPC_PORT as u32, "tcp", _HOST_RPC_PORT as u32)
                .build(),
        )
        .await;
    match docker_container {
        Ok(cci) => return cci.id,
        Err(e) => panic!("{}", e),
    };
}

#[cfg(feature = "docker")]
async fn docker_remove_container(docker: &Docker, container_id: &str) {
    docker
        .containers()
        .get(container_id)
        .remove(RmContainerOptions::builder().force(true).build())
        .await
        .unwrap()
}

#[cfg(feature = "docker")]
async fn docker_start_container(docker: &Docker, container_id: &str) {
    docker.containers().get(container_id).start().await.unwrap();
}
