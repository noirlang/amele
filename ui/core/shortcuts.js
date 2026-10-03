// Klavye Kısayolları ve Shift İpucu Yöneticisi (Keyboard Shortcuts & Shift Overlay Manager)
// Shift tuşuna basıldığında belirli div ve butonların yanında kısayol tuş rozetleri otomatik olarak belirir.
// Örneğin: Amele radyal menü yanında 'M' tuşu belirir, Shift + M ile radyal menü açılır/kapanır.

export function isTypingElement(target) {
  if (!target && typeof document !== "undefined") {
    target = document.activeElement;
  }
  if (!target) return false;
  const tagName = target.tagName ? target.tagName.toLowerCase() : "";
  if (tagName === "input" || tagName === "textarea" || tagName === "select") {
    return true;
  }
  if (target.isContentEditable || target.getAttribute?.("contenteditable") === "true") {
    return true;
  }
  if (typeof target.closest === "function" && target.closest("input, textarea, select, [contenteditable='true']")) {
    return true;
  }
  return false;
}

export function syncShortcutBadges() {
  if (typeof document === "undefined" || !document.querySelectorAll) return;
  const shortcutElements = document.querySelectorAll("[data-shortcut]");
  shortcutElements.forEach((el) => {
    const key = el.getAttribute("data-shortcut");
    if (!key) return;

    // Eğer buton dairesel menü sarıcısı içindeyse rozeti wrap'e iliştir
    const wrap = typeof el.closest === "function" ? el.closest(".nav-circle-btn-wrap") : null;
    const container = wrap || el;

    if (container.querySelector && !container.querySelector(".shortcut-key-badge")) {
      const badge = document.createElement("span");
      badge.className = "shortcut-key-badge";
      badge.setAttribute("aria-hidden", "true");
      badge.textContent = key.toUpperCase();
      if (typeof container.appendChild === "function") {
        container.appendChild(badge);
      }
    }
  });
}

export function initKeyboardShortcuts({
  state,
  t,
  toggleNavMenu,
  openNavMenu,
  closeNavMenu,
  setRoute,
  toggleCaseSidebar,
  toggleReportSidebar,
  logout
} = {}) {
  if (typeof window === "undefined" || typeof document === "undefined") {
    return {
      destroy: () => {},
      showHints: () => {},
      hideHints: () => {},
      syncBadges: () => {}
    };
  }

  let isShiftActive = false;

  const showHints = () => {
    if (isShiftActive) return;
    isShiftActive = true;
    syncShortcutBadges();
    document.body?.classList?.add?.("show-key-hints");
    document.documentElement?.classList?.add?.("show-key-hints");
  };

  const hideHints = () => {
    if (!isShiftActive && !document.body?.classList?.contains?.("show-key-hints")) return;
    isShiftActive = false;
    document.body?.classList?.remove?.("show-key-hints");
    document.documentElement?.classList?.remove?.("show-key-hints");
  };

  const handleKeyDown = (event) => {
    if (!event) return;

    // Kullanıcı bir metin giriş alanında yazı yazıyorsa kısayolları yutma
    if (isTypingElement(event.target)) return;

    // Ctrl, Alt, Meta (Cmd) basılıysa tarayıcı veya sistem kısayollarını engelleme
    if (event.ctrlKey || event.altKey || event.metaKey) return;

    // Shift tuşuna basıldığında otomatik ipucu tuşlarını göster
    if (event.key === "Shift" || event.code === "ShiftLeft" || event.code === "ShiftRight") {
      showHints();
      return;
    }

    // Escape basıldığında ipuçlarını ve açık menüyü kapat
    if (event.key === "Escape") {
      hideHints();
      return;
    }

    // Shift tuşu basılıyken bir tuşa basıldıysa ilgili kısayolu çalıştır
    if (event.shiftKey) {
      if (!isShiftActive) {
        showHints();
      }

      const rawKey = event.key || "";
      const key = rawKey.toUpperCase();

      // Amele Radyal Menü Aç/Kapat (Shift + M)
      if (key === "M") {
        event.preventDefault();
        event.stopPropagation();
        if (typeof toggleNavMenu === "function") {
          toggleNavMenu();
        }
        // Menü DOM'a çizildikten sonra Shift hâlâ basılıysa rozetleri güncelle
        setTimeout(() => {
          if (isShiftActive) {
            syncShortcutBadges();
          }
        }, 35);
        return;
      }

      // Sol Vaka Paneli Aç/Kapat (Shift + C)
      if (key === "C") {
        event.preventDefault();
        event.stopPropagation();
        if (typeof toggleCaseSidebar === "function") {
          toggleCaseSidebar();
        }
        return;
      }

      // Sağ Rapor Paneli Aç/Kapat (Shift + R)
      if (key === "R") {
        event.preventDefault();
        event.stopPropagation();
        if (typeof toggleReportSidebar === "function") {
          toggleReportSidebar();
        }
        return;
      }

      // Hızlı Navigasyon Rotaları (Shift + H, W, L, D, A, I, O, Y/?, B, S, P)
      const routeMap = {
        H: "home",
        W: "windows",
        L: "linux",
        D: "docker",
        A: "android",
        I: "ios",
        O: "other",
        Y: "help",
        "?": "help",
        B: "about",
        S: "settings",
        P: "profile"
      };

      if (routeMap[key]) {
        event.preventDefault();
        event.stopPropagation();
        if (typeof closeNavMenu === "function" && state?.navMenu?.isOpen) {
          closeNavMenu();
        }
        if (typeof setRoute === "function") {
          setRoute(routeMap[key]);
        }
        return;
      }

      // Oturumu Kapat / Profil Değiştir (Shift + Q)
      if (key === "Q") {
        event.preventDefault();
        event.stopPropagation();
        if (typeof closeNavMenu === "function" && state?.navMenu?.isOpen) {
          closeNavMenu();
        }
        if (typeof logout === "function") {
          logout();
        }
        return;
      }

      // Arayüzdeki herhangi bir [data-shortcut="KEY"] elemanını doğrudan tıkla
      if (typeof document.querySelector === "function") {
        const customEl = document.querySelector(`[data-shortcut="${key}" i]`);
        if (customEl && typeof customEl.click === "function") {
          event.preventDefault();
          event.stopPropagation();
          customEl.click();
          return;
        }
      }
    }
  };

  const handleKeyUp = (event) => {
    if (!event) return;
    if (event.key === "Shift" || !event.shiftKey) {
      hideHints();
    }
  };

  const handleBlur = () => {
    hideHints();
  };

  const handleVisibilityChange = () => {
    if (document.hidden) {
      hideHints();
    }
  };

  window.addEventListener("keydown", handleKeyDown, { capture: false });
  window.addEventListener("keyup", handleKeyUp, { capture: false });
  window.addEventListener("blur", handleBlur);
  if (typeof document.addEventListener === "function") {
    document.addEventListener("visibilitychange", handleVisibilityChange);
  }

  return {
    destroy: () => {
      window.removeEventListener("keydown", handleKeyDown);
      window.removeEventListener("keyup", handleKeyUp);
      window.removeEventListener("blur", handleBlur);
      if (typeof document.removeEventListener === "function") {
        document.removeEventListener("visibilitychange", handleVisibilityChange);
      }
      hideHints();
    },
    showHints,
    hideHints,
    syncBadges: syncShortcutBadges
  };
}
