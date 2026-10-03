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
    getElementById: () => null,
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

  await t.test("Shift + V and Shift + C toggle case sidebar, Shift + R toggles report sidebar", () => {
    let caseToggleCount = 0;
    let reportToggleCount = 0;

    const controller = initKeyboardShortcuts({
      toggleCaseSidebar: () => { caseToggleCount++; },
      toggleReportSidebar: () => { reportToggleCount++; }
    });

    window._dispatch("keydown", {
      key: "V",
      shiftKey: true,
      preventDefault: () => {},
      stopPropagation: () => {},
      target: { tagName: "BODY" }
    });
    assert.strictEqual(caseToggleCount, 1, "case sidebar should toggle on Shift+V");

    window._dispatch("keydown", {
      key: "C",
      shiftKey: true,
      preventDefault: () => {},
      stopPropagation: () => {},
      target: { tagName: "BODY" }
    });
    assert.strictEqual(caseToggleCount, 2, "case sidebar should also toggle on Shift+C (backward compatibility)");

    window._dispatch("keydown", {
      key: "R",
      shiftKey: true,
      preventDefault: () => {},
      stopPropagation: () => {},
      target: { tagName: "BODY" }
    });
    assert.strictEqual(reportToggleCount, 1, "report sidebar should toggle on Shift+R");

    controller.destroy();
  });

  await t.test("Shift + V followed by digit selects corresponding case", () => {
    let clickedCase = null;
    const mockCaseCard = {
      click: () => { clickedCase = "Case 2"; }
    };

    const origQuerySelector = document.querySelector;
    document.querySelector = (sel) => {
      if (sel === '.case-sidebar-card[data-shortcut="2"]') return mockCaseCard;
      return null;
    };

    const controller = initKeyboardShortcuts({
      toggleCaseSidebar: () => {}
    });

    // Press Shift + V
    window._dispatch("keydown", {
      key: "V",
      shiftKey: true,
      preventDefault: () => {},
      stopPropagation: () => {},
      target: { tagName: "BODY" }
    });

    // Press 2
    window._dispatch("keydown", {
      key: "2",
      code: "Digit2",
      shiftKey: false,
      preventDefault: () => {},
      stopPropagation: () => {},
      target: { tagName: "BODY" }
    });

    assert.strictEqual(clickedCase, "Case 2", "Case 2 should be clicked after Shift+V + 2 sequence");

    document.querySelector = origQuerySelector;
    controller.destroy();
  });

  await t.test("Shift + V followed by N or Shift + N triggers new case prompt", () => {
    let newCaseTriggered = false;
    const mockNewBtn = {
      click: () => { newCaseTriggered = true; }
    };

    const origQuerySelector = document.querySelector;
    document.querySelector = (sel) => {
      if (sel === '[data-action="new-case-prompt"]') return mockNewBtn;
      return null;
    };

    const controller = initKeyboardShortcuts({
      toggleCaseSidebar: () => {}
    });

    // Sequence: Shift + V then N
    window._dispatch("keydown", {
      key: "V",
      shiftKey: true,
      preventDefault: () => {},
      stopPropagation: () => {},
      target: { tagName: "BODY" }
    });

    window._dispatch("keydown", {
      key: "N",
      shiftKey: false,
      preventDefault: () => {},
      stopPropagation: () => {},
      target: { tagName: "BODY" }
    });

    assert.strictEqual(newCaseTriggered, true, "New case prompt should trigger on Shift+V then N");

    // Direct: Shift + N
    newCaseTriggered = false;
    window._dispatch("keydown", {
      key: "N",
      shiftKey: true,
      preventDefault: () => {},
      stopPropagation: () => {},
      target: { tagName: "BODY" }
    });

    assert.strictEqual(newCaseTriggered, true, "New case prompt should trigger on direct Shift+N");

    document.querySelector = origQuerySelector;
    controller.destroy();
  });

  await t.test("Shift + T toggles language", () => {
    let switchedLang = null;
    const controller = initKeyboardShortcuts({
      state: { language: "tr" },
      setLanguage: (lang) => { switchedLang = lang; }
    });

    window._dispatch("keydown", {
      key: "T",
      shiftKey: true,
      preventDefault: () => {},
      stopPropagation: () => {},
      target: { tagName: "BODY" }
    });

    assert.strictEqual(switchedLang, "en", "Language should toggle from tr to en on Shift+T");
    controller.destroy();
  });

  await t.test("Workflow shortcuts: Back, Scan, Approve Key, Reset Key, Connect, Start", () => {
    const clickedButtons = [];
    const origQuerySelector = document.querySelector;

    document.querySelector = (sel) => {
      const el = {
        click: () => clickedButtons.push(sel),
        classList: { contains: () => false },
        disabled: false,
        style: {}
      };
      if (sel === ".workflow-back-btn") return el;
      if (sel === '.workflow-target-actions [data-action="scan"]') return el;
      if (sel === '[data-action="approve-key"]') return el;
      if (sel === '[data-action="reset-key"]') return el;
      if (sel === '[data-action="connect"]') return el;
      if (sel === '.workflow-target-actions [data-action="start"]:not([hidden]):not([disabled])') return el;
      if (sel === '[data-action="check-update"]') return el;
      return null;
    };

    const controller = initKeyboardShortcuts();

    // Shift + B -> Back
    window._dispatch("keydown", { key: "B", shiftKey: true, preventDefault: () => {}, stopPropagation: () => {}, target: { tagName: "BODY" } });
    assert.strictEqual(clickedButtons.includes(".workflow-back-btn"), true, "Shift+B should click back button");

    // Shift + S -> Scan
    window._dispatch("keydown", { key: "S", shiftKey: true, preventDefault: () => {}, stopPropagation: () => {}, target: { tagName: "BODY" } });
    assert.strictEqual(clickedButtons.includes('.workflow-target-actions [data-action="scan"]'), true, "Shift+S should click scan");

    // Shift + K -> Approve Key
    window._dispatch("keydown", { key: "K", shiftKey: true, preventDefault: () => {}, stopPropagation: () => {}, target: { tagName: "BODY" } });
    assert.strictEqual(clickedButtons.includes('[data-action="approve-key"]'), true, "Shift+K should click approve-key");

    // Shift + X -> Reset Key
    window._dispatch("keydown", { key: "X", shiftKey: true, preventDefault: () => {}, stopPropagation: () => {}, target: { tagName: "BODY" } });
    assert.strictEqual(clickedButtons.includes('[data-action="reset-key"]'), true, "Shift+X should click reset-key");

    // Enter -> Connect
    window._dispatch("keydown", { key: "Enter", shiftKey: false, preventDefault: () => {}, stopPropagation: () => {}, target: { tagName: "BODY" } });
    assert.strictEqual(clickedButtons.includes('[data-action="connect"]'), true, "Enter should click connect");

    // Shift + E -> Start
    window._dispatch("keydown", { key: "E", shiftKey: true, preventDefault: () => {}, stopPropagation: () => {}, target: { tagName: "BODY" } });
    assert.strictEqual(clickedButtons.includes('.workflow-target-actions [data-action="start"]:not([hidden]):not([disabled])'), true, "Shift+E should click start");

    // Shift + U -> Check Update
    window._dispatch("keydown", { key: "U", shiftKey: true, preventDefault: () => {}, stopPropagation: () => {}, target: { tagName: "BODY" } });
    assert.strictEqual(clickedButtons.includes('[data-action="check-update"]'), true, "Shift+U should click check-update");

    document.querySelector = origQuerySelector;
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
