FROM rust:1-buster
ENV PATH=/root/.cache/.wasm-pack/.wasm-bindgen-cargo-install-0.2.83/bin:$PATH

RUN cargo install typeshare-cli
RUN curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh

COPY . .

WORKDIR wasm-lib
RUN wasm-pack build --release --target web
RUN typeshare . --lang typescript --output-file pkg/structTypes.ts