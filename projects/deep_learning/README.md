# MNIST-rs

This demo project demonstrates how to train and perform inference in TEE.

## Install the TAs

There are two TAs in the project:

| TA | UUID | Usage |
| ---- | ---- | ---- |
| Train | 1b5f5b74-e9cf-4e62-8c3e-7e41da6d76f6 | for training new Model|
| Inference | ff09aa8a-fbb9-4734-ae8c-d7cd1a3f6744 | for performing reference|

The `Train TA` consumes more memory than `Inference TA`.

Make sure to install them before attempting to use any functions.

## Running the Host

There are three subcommands in the host:

1. Train

    Trains a new model and exports it to the given path.

    ``` shell
    cargo run -- train
    ```

    This subcommand downloads the MNIST dataset, trains a new model, and outputs
    the model to the given path(default: `model.bin`).

    For detailed usage, run: `cargo run -- train --help`.

2. Infer

    Loads a model from the given path, tests it with a given image, and prints
    the inference result.

    ```shell
    # cargo run -- infer -b samples/0.bin -b samples/1.bin -i samples/7.png
    cargo run -- infer [-b ${binary_path} | -i ${image_path}]
    ```

    This subcommand loads the model the model from the given
    path(default: `model.bin`) and tests it with the given binaries and images,
    and prints the inference results. For convenience, you can use the sample
    binaries and images in the `samples` folder.

    For detailed usage, run: `cargo run -- infer --help`.

3. Serve

    Loads a model from the given path, starts a web server and serves it as an
    API.

    ```shell
    cargo run -- serve
    ```

    This subcommand loads the model the model from the given
    path(default: `model.bin`) and starts a web server to provide inference
    APIs.

    **Available APIs**:

    | Method | Endpoint | Body |
    | ---- | ---- | ---- |
    | POST | `/inference/image` | an image with dimensions 28x28 |
    | POST | `/inference/binary` | a 784-byte binary |

    You can test the server with the following commands:

    ```shell
    # Perform inference using an image
    curl --data-binary "@./samples/7.png" http://localhost:3000/inference/image
    # Perform inference using a binary file
    curl --data-binary "@./samples/7.bin" http://localhost:3000/inference/binary
    ```

    For detailed usage, run: `cargo run -- serve --help`.

## Credits

This demo project is inspired by the crates and examples from
[tracel-ai/burn](https://github.com/tracel-ai/burn), including:

1. [crates/burn-no-std-tests](https://github.com/tracel-ai/burn/tree/v0.16.0/crates/burn-dataset)
2. [examples/custom-training-loop](https://github.com/tracel-ai/burn/tree/v0.16.0/examples/custom-training-loop)
3. [examples/mnist-inference-web](https://github.com/tracel-ai/burn/tree/v0.16.0/examples/mnist-inference-web)

Special thanks to @[Guillaume Lagrange](https://github.com/laggui) for sharing
knowledge and providing early reviews.
