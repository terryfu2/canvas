const { createProxyMiddleware } = require("http-proxy-middleware");

module.exports = function(app) {
    app.use(
        "/api",
        createProxyMiddleware({
        target: `http://${process.env.REACT_APP_BACKEND_HOST}:${process.env.REACT_APP_BACKEND_PORT}`,
        pathRewrite: { "^/api": "" }
        })
    );
    app.use(
      "/ws_api",
      createProxyMiddleware({
      target: `ws://${process.env.REACT_APP_BACKEND_HOST}:${process.env.REACT_APP_BACKEND_PORT}`,
      pathRewrite: { "^/ws_api": "" }
      })
  );

};
