const test = require("node:test");
const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const protocol = require("./protocol.js");

const golden = JSON.parse(fs.readFileSync(path.join(__dirname, "protocol.golden.json"), "utf8"));

test("protocol constants match the checked-in golden values", () => {
    for (const [name, value] of Object.entries(golden.constants)) {
        assert.deepEqual(protocol[name], value, name);
    }
});

test("protocol headers and endpoints match the checked-in golden values", () => {
    assert.equal(protocol.SESSION_HEADER, golden.headers.session);
    assert.equal(protocol.CLIENT_KEY_HEADER, golden.headers.clientKey);
    assert.deepEqual({ ...protocol.ENDPOINTS }, golden.endpoints);
});

test("httpsUrl preserves host, port and path", () => {
    assert.equal(protocol.httpsUrl("192.168.1.5", 8790, "/client/"), "https://192.168.1.5:8790/client/");
    assert.equal(protocol.httpsUrl("fe80::1", 8790, "/x"), "https://[fe80::1]:8790/x");
});
