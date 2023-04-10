import("./pkg").then((module) => {
  module.default().then((mod) => {
    module.start_app("gui");
  });
});
