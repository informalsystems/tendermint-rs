use crate::prelude::*;

pub trait Scheduler {
    fn schedule(
        &self,
        trusted_store: &StoreReader<Trusted>,
        next_height: Height,
        target_height: Height,
    ) -> Height;
}

impl<F> Scheduler for F
where
    F: Fn(&StoreReader<Trusted>, Height, Height) -> Height,
{
    fn schedule(
        &self,
        trusted_store: &StoreReader<Trusted>,
        next_height: Height,
        target_height: Height,
    ) -> Height {
        self(trusted_store, next_height, target_height)
    }
}

pub fn schedule(
    trusted_store: &StoreReader<Trusted>,
    next_height: Height,
    target_height: Height,
) -> Height {
    precondition!(trusted_store.highest_height().is_some());

    let latest_trusted_height = trusted_store.highest_height().unwrap();

    if latest_trusted_height < next_height && latest_trusted_height < target_height {
        return middle_point(latest_trusted_height, next_height);
    }

    if latest_trusted_height == next_height && latest_trusted_height < target_height {
        return target_height;
    }

    if latest_trusted_height == target_height {
        return target_height;
    }

    middle_point(next_height, target_height)
}

fn middle_point(low: Height, high: Height) -> Height {
    precondition!(low < high);
    let result = low + (high + 1 - low) / 2;
    postcondition!(low < result && result <= high);
    result
}
