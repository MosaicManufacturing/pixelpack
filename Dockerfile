FROM rust:1-buster
ENV PATH=/root/.cache/.wasm-pack/.wasm-bindgen-cargo-install-0.2.83/bin:$PATH

COPY . .

WORKDIR wasm-lib

RUN cargo install typeshare-cli && \
    curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh && \
    wasm-pack build --release --target web && \
    typeshare . --lang typescript --output-file pkg/structTypes.ts
