import os from "os";

// import foreign from "./native/libimager_nodejs.apple.node";
const imager_native = function(){
    let platform = os.platform();
    console.assert(
        platform === "darwin" ||
        platform === "linux" ||
        platform === "win32"
    );
    let apple_path = "./native/libimager_nodejs.apple.node";
    let linux_path = "./native/libimager_nodejs.linux.node";
    let windows_path = "./native/libimager_nodejs.windows.node";
    let unknown_platform = (): string => {
        throw "unknown platform";
    };
    let active_path = (platform === "darwin") ? apple_path
        : (platform === "linux") ? linux_path
        : (platform === "win32") ? windows_path
        : unknown_platform();
    return require(active_path);
}();


// console.log("module: ", imager_native);
// imager_native
//     .version()
//     .then((res : any) => console.log("result: ", res));

const input_path = "assets/samples/small/low/2yV-pyOxnPw300.jpeg";

imager_native
    .buffer_open(input_path)
    .then((buffer: any) => {
        return imager_native.buffer_opt(buffer, "full");
    })
    .then((buffer: any) => {
        return imager_native.buffer_save(buffer, "test.jpeg");
    });

// imager_native
//     .opt_from_file("../assets/samples/small/low/2yV-pyOxnPw300.jpeg", "full")
//     .then((x: Array<u>) => {
//         console.log("done");
//     });