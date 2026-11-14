(function (root, factory) {
    if (typeof module === "object" && module.exports) {
        module.exports = factory();
    } else {
        root.OrbiProtocol = factory();
    }
}(typeof self !== "undefined" ? self : this, function () {
    const TYPE_VIDEO = 1;
    const TYPE_HELLO = 2;
    const TYPE_HELLO_ACK = 3;
    const TYPE_PING = 4;
    const TYPE_PONG = 5;
    const TYPE_IDR = 6;
    const TYPE_BYE = 10;

    const MAX_FRAME = 4 * 1024 * 1024;
    const VIDEO_PREFIX = 18;
    const DATAGRAM_HEADER = 24;
    const DEFAULT_DATAGRAM = 1024;

    const FEC_BLOCK_FLAG = 0x02;
    const PARITY_LADDER = Object.freeze([0, 2, 3, 4]);

    const SESSION_HEADER = "x-orbiscreen-session";
    const CLIENT_KEY_HEADER = "x-orbiscreen-client-key";

    const ENDPOINTS = Object.freeze({
        pairStatus: "/api/pair/status",
        pairRequest: "/api/pair/request",
        pair: "/api/pair",
        pairApprove: "/api/pair/approve",
        pairDeny: "/api/pair/deny",
        control: "/api/control",
        input: "/input",
        au: "/au",
        idr: "/idr",
        session: "/api/session",
        info: "/api/info",
        clientConfig: "/client/config.json",
    });

    function isHttps() {
        return typeof window !== "undefined"
            && !!window.location
            && window.location.protocol === "https:";
    }

    function httpsUrl(host, port, path) {
        let name = String(host || "").trim() || "host";
        if (name.includes(":") && !name.startsWith("[")) name = `[${name}]`;
        const portPart = port ? `:${port}` : "";
        const suffix = path ? (path.startsWith("/") ? path : `/${path}`) : "/";
        return `https://${name}${portPart}${suffix}`;
    }

    return {
        TYPE_VIDEO, TYPE_HELLO, TYPE_HELLO_ACK, TYPE_PING, TYPE_PONG, TYPE_IDR, TYPE_BYE,
        MAX_FRAME, VIDEO_PREFIX, DATAGRAM_HEADER, DEFAULT_DATAGRAM,
        FEC_BLOCK_FLAG, PARITY_LADDER,
        SESSION_HEADER, CLIENT_KEY_HEADER, ENDPOINTS,
        isHttps, httpsUrl,
    };
}));
