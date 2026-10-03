import test from "node:test";
import assert from "node:assert";

// Mock minimal DOM environment if running in pure Node.js
if (typeof globalThis.document === "undefined") {
  const classListSet = new Set();
  const mockClassList = {
    add: (cls) => classListSet.add(cls),
    remove: (cls) => classListSet.delete(cls),
    contains: (cls) => classListSet.has(cls),
    toggle: (cls) => (classListSet.has(cls) ? classListSet.delete(cls) : classListSet.add(cls))
  };

  const createMockElement = (tagName = "div", attributes = {}) => {
    const attrs = { ...attributes };
    const children = [];
    const el = {
      tagName: tagName.toUpperCase(),
      classList: {
        add: () => {},
        remove: () => {},
        contains: () => false,
        toggle: () => {}
      },
      attributes: attrs,
      getAttribute: (k) => attrs[k] ?? null,
      setAttribute: (k, v) => { attrs[k] = String(v); },
      hasAttribute: (k) => k in attrs,
      closest: (sel) => {
        if (sel === ".nav-circle-btn-wrap" && attrs["data-is-wrapped"]) {
          return createMockElement("div", { class: "nav-circle-btn-wrap" });
        }
        return null;
      },
      querySelector: (sel) => children.find((c) => c._selector === sel) || null,
      querySelectorAll: (sel) => children.filter((c) => c._selector === sel),
      appendChild: (child) => {
        children.push(child);
        return child;
      },
      clickCount: 0,
      click: () => { el.clickCount++; }
    };
    return el;
  };

  const windowListeners = new Map();
  const documentListeners = new Map();

  globalThis.window = {
    addEventListener: (ev, fn) => {
      if (!windowListeners.has(ev)) windowListeners.set(ev, []);
      windowListeners.get(ev).push(fn);
    },
    removeEventListener: (ev, fn) => {
      if (!windowListeners.has(ev)) return;
      windowListeners.set(ev, windowListeners.get(ev).filter((f) => f !== fn));
    },
    _dispatch: (ev, eventObj) => {
      (windowListeners.get(ev) || []).forEach((fn) => fn(eventObj));
    }
  };

  globalThis.document = {
    body: {
      classList: mockClassList
    },
    documentElement: {
      classList: mockClassList
    },
    createElement: (tag) => {
      const el = createMockElement(tag);
      el._selector = `.${tag}`;
      return el;
    },
    querySelector: () => null,
    querySelectorAll: () => [],
    addEventListener: (ev, fn) => {
      if (!documentListeners.has(ev)) documentListeners.set(ev, []);
      documentListeners.get(ev).push(fn);
    },
    removeEventListener: (ev, fn) => {
      if (!documentListeners.has(ev)) return;
      documentListeners.set(ev, documentListeners.get(ev).filter((f) => f !== fn));
    },
    _dispatch: (ev, eventObj) => {
      (documentListeners.get(ev) || []).forEach((fn) => fn(eventObj));
    }
  };
}

test("Keyboard Shortcuts System", async (t) => {
  const { isTypingElement, initKeyboardShortcuts, syncShortcutBadges } = await import("../ui/core/shortcuts.js");

  await t.test("isTypingElement correctly identifies input elements", () => {
    assert.strictEqual(isTypingElement({ tagName: "INPUT" }), true);
    assert.strictEqual(isTypingElement({ tagName: "TEXTAREA" }), true);
    assert.strictEqual(isTypingElement({ tagName: "SELECT" }), true);
    assert.strictEqual(isTypingElement({ tagName: "DIV", isContentEditable: true }), true);
    assert.strictEqual(isTypingElement({
      tagName: "SPAN",
      closest: (sel) => (sel.includes("input") ? { tagName: "INPUT" } : null)
    }), true);
    assert.strictEqual(isTypingElement({ tagName: "DIV", closest: () => null }), false);
    assert.strictEqual(isTypingElement({ tagName: "BUTTON", closest: () => null }), false);
    assert.strictEqual(isTypingElement(null), false);
  });

  await t.test("Shift keydown activates hints and keyup deactivates hints", () => {
    const controller = initKeyboardShortcuts();

    // Trigger Shift keydown
    window._dispatch("keydown", {
      key: "Shift",
      code: "ShiftLeft",
      target: { tagName: "BODY" }
    });
    assert.strictEqual(document.body.classList.contains("show-key-hints"), true);

    // Trigger Shift keyup
    window._dispatch("keyup", {
      key: "Shift",
      shiftKey: false,
      target: { tagName: "BODY" }
    });
    assert.strictEqual(document.body.classList.contains("show-key-hints"), false);

    controller.destroy();
  });

  await t.test("Shift + M toggles Amele radial menu", () => {
    let toggleCount = 0;
    let prevented = false;

    const controller = initKeyboardShortcuts({
      toggleNavMenu: () => { toggleCount++; }
    });

    window._dispatch("keydown", {
      key: "M",
      shiftKey: true,
      preventDefault: () => { prevented = true; },
      stopPropagation: () => {},
      target: { tagName: "BUTTON" }
    });

    assert.strictEqual(toggleCount, 1, "toggleNavMenu should have been called once");
    assert.strictEqual(prevented, true, "default should be prevented");

    controller.destroy();
  });

  await t.test("Shift + H, W, L, D navigate to respective routes", () => {
    const navigatedRoutes = [];
    let menuClosed = false;

    const controller = initKeyboardShortcuts({
      state: { navMenu: { isOpen: true } },
      closeNavMenu: () => { menuClosed = true; },
      setRoute: (r) => { navigatedRoutes.push(r); }
    });

    const testKeys = [
      { key: "H", expected: "home" },
      { key: "W", expected: "windows" },
      { key: "L", expected: "linux" },
      { key: "D", expected: "docker" },
      { key: "A", expected: "android" },
      { key: "I", expected: "ios" },
      { key: "O", expected: "other" },
      { key: "Y", expected: "help" },
      { key: "B", expected: "about" },
      { key: "S", expected: "settings" },
      { key: "P", expected: "profile" }
    ];

    testKeys.forEach(({ key, expected }) => {
      window._dispatch("keydown", {
        key,
        shiftKey: true,
        preventDefault: () => {},
        stopPropagation: () => {},
        target: { tagName: "BODY" }
      });
      assert.strictEqual(navigatedRoutes[navigatedRoutes.length - 1], expected);
    });

    assert.strictEqual(menuClosed, true, "radial menu should be closed upon navigation");
    controller.destroy();
  });

  await t.test("Shift + C and Shift + R toggle sidebars", () => {
    let caseToggled = false;
    let reportToggled = false;

    const controller = initKeyboardShortcuts({
      toggleCaseSidebar: () => { caseToggled = true; },
      toggleReportSidebar: () => { reportToggled = true; }
    });

    window._dispatch("keydown", {
      key: "C",
      shiftKey: true,
      preventDefault: () => {},
      stopPropagation: () => {},
      target: { tagName: "BODY" }
    });
    assert.strictEqual(caseToggled, true, "case sidebar should toggle on Shift+C");

    window._dispatch("keydown", {
      key: "R",
      shiftKey: true,
      preventDefault: () => {},
      stopPropagation: () => {},
      target: { tagName: "BODY" }
    });
    assert.strictEqual(reportToggled, true, "report sidebar should toggle on Shift+R");

    controller.destroy();
  });

  await t.test("Typing in an input with Shift does not trigger shortcuts", () => {
    let toggleCount = 0;

    const controller = initKeyboardShortcuts({
      toggleNavMenu: () => { toggleCount++; }
    });

    // Simulate typing an uppercase 'M' in an input field
    window._dispatch("keydown", {
      key: "M",
      shiftKey: true,
      preventDefault: () => {},
      stopPropagation: () => {},
      target: {
        tagName: "INPUT",
        closest: (sel) => (sel.includes("input") ? { tagName: "INPUT" } : null)
      }
    });

    assert.strictEqual(toggleCount, 0, "shortcut must NOT trigger when typing in an input");
    controller.destroy();
  });

  await t.test("System modifier keys (Ctrl/Meta/Alt) are ignored", () => {
    let toggleCount = 0;

    const controller = initKeyboardShortcuts({
      toggleNavMenu: () => { toggleCount++; }
    });

    window._dispatch("keydown", {
      key: "M",
      shiftKey: true,
      ctrlKey: true,
      preventDefault: () => {},
      stopPropagation: () => {},
      target: { tagName: "BODY" }
    });

    assert.strictEqual(toggleCount, 0, "Ctrl+Shift+M must be ignored by shortcuts manager");
    controller.destroy();
  });

  await t.test("Digit shortcuts (1..9) trigger forensic card clicks directly and with Shift", () => {
    let clicked = false;
    const mockCard = {
      classList: {
        contains: (cls) => cls === "is-disabled" ? false : false
      },
      disabled: false,
      click: () => { clicked = true; }
    };

    const origQuerySelector = document.querySelector;
    document.querySelector = (sel) => {
      if (sel === '[data-shortcut="1"]') return mockCard;
      return null;
    };

    const controller = initKeyboardShortcuts();

    // With Shift + 1 (code: Digit1, key: !)
    window._dispatch("keydown", {
      key: "!",
      code: "Digit1",
      shiftKey: true,
      preventDefault: () => {},
      stopPropagation: () => {},
      target: { tagName: "BODY" }
    });
    assert.strictEqual(clicked, true, "Digit1 with Shift should trigger click on card 1");

    // Reset and test without Shift (press 1 directly)
    clicked = false;
    window._dispatch("keydown", {
      key: "1",
      code: "Digit1",
      shiftKey: false,
      preventDefault: () => {},
      stopPropagation: () => {},
      target: { tagName: "BODY" }
    });
    assert.strictEqual(clicked, true, "1 key directly should trigger click on card 1");

    document.querySelector = origQuerySelector;
    controller.destroy();
  });
});
