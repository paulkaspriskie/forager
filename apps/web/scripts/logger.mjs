export const watchLogger = {
  name: "watch-logger",

  setup(build) {
    build.onStart(() => {
      console.log("\n[build] Changes detected. Rebuilding...");
    });

    build.onEnd((result) => {
      if (result.errors.length > 0) {
        console.error("[build] Build failed.");
      } else {
        console.log("[build] Build completed successfully.");
      }
    });
  },
};