{
  "targets": [{
    "target_name": "imager_napi",
    "conditions": [
      ['OS=="mac"', {"libraries": [
        "../../../target/x86_64-apple-darwin/release/libimager_nodejs.dylib"
      ]}],

      # FIX LATER:
      # ['OS=="linux" or OS=="solaris" or OS=="freebsd"', {"libraries": [
      #   "../../../target/x86_64-unknown-linux-gnu/release/libimager_nodejs.so"
      # ]}],

      # MAYBE TODO:
      # ['OS=="win"', {"libraries": ["../../target/release/libimager_nodejs.lib"]}]
    ],
  }]
}
