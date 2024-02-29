const { createProxyMiddleware } = require("http-proxy-middleware");

module.exports = function(app) {
    app.use(
        "/api",
        createProxyMiddleware({
        target: `http://${process.env.REACT_APP_HTTP_HOST}:${process.env.REACT_APP_HTTP_PORT}`,
        pathRewrite: { "^/api": "" }
        })
    );


};
