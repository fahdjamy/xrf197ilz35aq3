# xrf197ilz35aq3

#### SETUP

1. On Mac, make sure you installed C++ driver
    1. `brew install cassandra-cpp-driver libuv openssl pkg-config`
    2. find the C++ library and headers located in the build.rs script of the crate cassandra-cpp-sys which
       cassandra-cpp likely depends on.
        1. As of the current build (`3.0.2`)
           check [cassandra-sys-rs](https://github.com/cassandra-rs/cassandra-sys-rs/tree/main) -> [build.rs](https://github.com/cassandra-rs/cassandra-sys-rs/blob/main/build.rs)
        2. It is using CASSANDRA_SYS_LIB_PATH so export the following.
            1. `export CPATH="$(brew --prefix)/include:$CPATH"` To tell the linker where libcassandra.dylib is.
            2. `export LIBRARY_PATH="$(brew --prefix)/lib:$LIBRARY_PATH"` To tell the C compiler (used by build.rs)
               where
               headers are.
    3. Set OpenSSL Variables: Because the build script does check for these.
        1. `export OPENSSL_ROOT_DIR="$(brew --prefix openssl)"`
        2. `export OPENSSL_LIB_DIR="$(brew --prefix openssl)/lib"`
        3. `export OPENSSL_INCLUDE_DIR="$(brew --prefix openssl)/include"`
