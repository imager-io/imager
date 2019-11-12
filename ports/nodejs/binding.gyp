{
  "targets": [{
    "target_name": "imager_napi",
    "conditions": [
      ['OS=="linux" or OS=="solaris" or OS=="freebsd"', {"libraries": [
        "../../../target/x86_64-unknown-linux-gnu/release/libimager_nodejs.so"
      ]}],
      ['OS=="mac"', {"libraries": [
        "../../../target/x86_64-apple-darwin/release/libimager_nodejs.dylib"
      ]}],
      # ['OS=="win"', {"libraries": ["../../target/release/libimager_nodejs.lib"]}]
    ],
  }]
}
