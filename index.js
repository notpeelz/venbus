const { platform, arch } = require("node:process");

let nativeBinding = null;

switch (platform) {
    case "linux":
        switch (arch) {
            case "x64":
                nativeBinding = require("./venbus.linux-x64-gnu.node");
                break;
            case "arm64":
                nativeBinding = require("./venbus.linux-arm64-gnu.node");
                break;
            default:
                throw new Error(`Unsupported architecture on Linux: ${arch}`);
        }
        break;
    default:
        throw new Error(`Unsupported OS: ${platform}, architecture: ${arch}`);
}

if (!nativeBinding) {
    throw new Error("Failed to load native binding");
}

module.exports = nativeBinding;
