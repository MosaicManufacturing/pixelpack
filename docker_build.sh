docker build -t compile_image .
docker run -v $PWD:/opt/mount --rm -ti compile_image bash -c "cp -r /wasm-lib/pkg /opt/mount/"
