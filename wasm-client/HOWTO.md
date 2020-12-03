# HOWTO

1. Install `wasm-pack`: https://rustwasm.github.io/wasm-pack/installer/

2. To just build the WASM library
   ```
   wasm-client/ $ wasm-pack build
   ```

3. Install npm dependencies
  ```
  wasm-client/app $ npm install
  ```

4. Serve the app
  ```
  wasm-client/app $ npm start
  ```

5. Open the app and look at the console
   ```
   open http://localhost:8080
   ```

Full tutorial: https://rustwasm.github.io/docs/wasm-pack/tutorials/npm-browser-packages/index.html

