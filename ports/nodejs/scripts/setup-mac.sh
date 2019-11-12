# CROSS-COMPILE - LINUX - DEPENDENCIES
brew install FiloSottile/musl-cross/musl-cross
brew tap SergioBenitez/osxct
brew install filosottile/musl-cross/musl-cross
brew install gcc

# CROSS-COMPILE - Linux
rustup target add i686-unknown-linux-gnu
rustup target add x86_64-unknown-linux-gnu

