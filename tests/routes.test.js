// api rotalarının düzgün çalışıp çalışmadığını kontrol eden test.

import test from "node:test";
import assert from "node:assert";

// Mock Browser environment for import testing
const createMockElement = () => {
  const el = {
    classList: {
      add: () => {},
      toggle: () => {},
      remove: () => {}
    },
    querySelectorAll: () => [],
    querySelector: () => el,
    addEventListener: () => {},
    focus: () => {},
    dataset: {},
    appendChild: (child) => child,
    removeChild: (child) => child,
    setAttribute: () => {},
    getAttribute: () => null,
    style: {},
    set innerHTML(val) {},
    get innerHTML() { return ""; }
  };
  return el;
};

const mockElement = createMockElement();

let clickListener = null;

const origSetTimeout = globalThis.setTimeout;
const unrefSetTimeout = (...args) => {
  const tid = origSetTimeout(...args);
  if (typeof tid?.unref === "function") tid.unref();
  return tid;
};
globalThis.setTimeout = unrefSetTimeout;

globalThis.window = {
  location: {
    origin: "http://127.0.0.1:8080",
    protocol: "http:",
    host: "127.0.0.1:8080",
    search: "?native=0&route=home"
  },
  addEventListener: () => {},
  clearTimeout: (...args) => globalThis.clearTimeout(...args),
  setTimeout: unrefSetTimeout
};
globalThis.location = globalThis.window.location;
globalThis.fetch = async (url) => {
  const dummy = { ok: true, agents: [], jobs: {} };
  return {
    ok: true,
    status: 200,
    text: async () => JSON.stringify(dummy),
    json: async () => dummy,
    clone: () => ({
      text: async () => JSON.stringify(dummy)
    })
  };
};
globalThis.window.fetch = globalThis.fetch;
if (typeof globalThis.navigator === "undefined") {
  globalThis.navigator = { userAgent: "Mozilla/5.0 (X11; Linux x86_64)", platform: "Linux x86_64" };
} else {
  try {
    Object.defineProperty(globalThis.navigator, "userAgent", { value: "Mozilla/5.0 (X11; Linux x86_64)", configurable: true });
    Object.defineProperty(globalThis.navigator, "platform", { value: "Linux x86_64", configurable: true });
  } catch (_) {}
}
globalThis.document = {
  documentElement: {
    classList: {
      add: () => {},
      toggle: () => {},
      remove: () => {}
    },
    lang: "tr",
    set lang(v) {}
  },
  body: {
    appendChild: () => {}
  },
  createElement: () => mockElement,
  querySelector: () => mockElement,
  querySelectorAll: () => [],
  getElementById: () => mockElement,
  addEventListener: (event, callback) => {
    if (event === "click") {
      clickListener = callback;
    }
  }
};
globalThis.localStorage = {
  getItem: () => "tr",
  setItem: () => {}
};

test("Frontend Routing and Module Health", async (t) => {
  await t.test("pages modules can be imported and expose expected functions", async () => {
    const { homePage, metric } = await import("../ui/pages/home.js");
    assert.strictEqual(typeof homePage, "function", "homePage should be a function");
    assert.strictEqual(typeof metric, "function", "metric should be a function");

    const { workflowPage, pickerField, field, pageTitle } = await import("../ui/pages/workflow.js");
    assert.strictEqual(typeof workflowPage, "function", "workflowPage should be a function");
    assert.strictEqual(typeof field, "function", "field should be a function");

    const { androidPage, androidModePage } = await import("../ui/android.js");
    assert.strictEqual(typeof androidPage, "function", "androidPage should be a function");
    assert.strictEqual(typeof androidModePage, "function", "androidModePage should be a function");

    const { iosPage, handleIosAction } = await import("../ui/ios.js");
    assert.strictEqual(typeof iosPage, "function", "iosPage should be a function");
    assert.strictEqual(typeof handleIosAction, "function", "handleIosAction should be a function");

    const { dockerPage } = await import("../ui/docker.js");
    assert.strictEqual(typeof dockerPage, "function", "dockerPage should be a function");

    const { toolsPage } = await import("../ui/pages/tools.js");
    assert.strictEqual(typeof toolsPage, "function", "toolsPage should be a function");

    const { helpPage } = await import("../ui/pages/help.js");
    assert.strictEqual(typeof helpPage, "function", "helpPage should be a function");

    const { otherPage, detailPanel } = await import("../ui/pages/other.js");
    assert.strictEqual(typeof otherPage, "function", "otherPage should be a function");
    assert.strictEqual(typeof detailPanel, "function", "detailPanel should be a function");

    const { renderReportSidebar } = await import("../ui/pages/reportSidebar.js");
    assert.strictEqual(typeof renderReportSidebar, "function", "renderReportSidebar should be a function");

    const { remoteAcqPage } = await import("../ui/tools/remote-acq/index.js");
    assert.strictEqual(typeof remoteAcqPage, "function", "remoteAcqPage should be a function");
  });

  await t.test("icons module correctly hydrated and exports functions", async () => {
    const { icon, hydrateIcons, icons } = await import("../ui/icons.js");
    assert.strictEqual(typeof icon, "function", "icon should be a function");
    assert.strictEqual(typeof hydrateIcons, "function", "hydrateIcons should be a function");
    assert.ok(icons.home, "home icon path should exist");
    assert.ok(icons.mouse, "mouse icon path should exist");
    assert.ok(icons["external-link"], "external-link icon path should exist");

    const { homePage } = await import("../ui/pages/home.js");
    const rendered = homePage({
      t: (k) => k,
      icon,
      assetPath: "/assets",
      theme: "dark",
      state: {
        news: [{ id: "n1", slug: "amele-v0-0-19", title: "Test Announcement" }]
      }
    });
    assert.ok(rendered.includes("news-h-title"), "news title should be rendered");
    assert.ok(!rendered.includes("data-news-h-scroll"), "scroll buttons should not be rendered");
    assert.ok(rendered.includes("data-news-link"), "data-news-link attribute should be present");
    assert.ok(rendered.includes("amele.noirlang.tr/news/amele-v0-0-19"), "link should point to news url");
  });

  await t.test("app.js initializes and executes without crashing", async () => {
    // This will import and immediately run the initialization scripts
    const appModule = await import("../ui/app.js");
    assert.ok(appModule, "app.js should be successfully imported");
  });

  await t.test("simulate navigation clicks through all routes without crashes", async () => {
    assert.ok(clickListener, "Click listener should be registered on document");

    const routesList = [
      "home",
      "tools",
      "windows",
      "linux",
      "android",
      "ios",
      "docker",
      "help",
      "analysis",
      "profile",
      "remote-acq",
      "other",
      "settings",
      "about",
      "workflow:windows-remote-disk",
      "workflow:linux-local-disk",
      "workflow:windows-local-ram",
      "workflow:linux-remote-ram",
      "android:logical",
      "android:filesystem",
      "android:ram",
      "android:diagnostics",
      "android:mft"
    ];

    for (const route of routesList) {
      const mockEvent = {
        preventDefault: () => {},
        target: {
          closest: (selector) => {
            if (selector === "[data-route]") {
              return { dataset: { route } };
            }
            return null;
          }
        }
      };

      // This will invoke setRoute and render() in app.js
      assert.doesNotThrow(() => {
        clickListener(mockEvent);
      }, `Route "${route}" should navigate and render without throwing exceptions`);
    }
  });

  await t.test("developer panel functions can be invoked without crashing", async () => {
    const { openDevPanel, closeDevPanel, toggleDevPanel, handleDevTrigger, devLog } = await import("../ui/developer.js");
    assert.strictEqual(typeof openDevPanel, "function", "openDevPanel should be a function");
    assert.strictEqual(typeof closeDevPanel, "function", "closeDevPanel should be a function");
    assert.strictEqual(typeof toggleDevPanel, "function", "toggleDevPanel should be a function");
    assert.strictEqual(typeof handleDevTrigger, "function", "handleDevTrigger should be a function");
    assert.strictEqual(typeof devLog, "function", "devLog should be a function");

    assert.doesNotThrow(() => {
      devLog("info", "test-scope", "test message", () => Promise.resolve({}), () => false);
      handleDevTrigger(() => Promise.resolve({}), () => false);
    });
  });
});
