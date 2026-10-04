// Klavye Kısayolları ve Shift İpucu Yöneticisi (Keyboard Shortcuts & Shift Overlay Manager)
// Shift tuşuna basıldığında belirli div ve butonların yanında kısayol tuş rozetleri otomatik olarak belirir.
// Örneğin: Amele radyal menü yanında 'M' tuşu belirir, Shift + M ile radyal menü açılır/kapanır.
// Vaka menüsü Shift + V ile açılır; V + [1-9] veya açıkken [1-9] vaka seçer; Shift + V + N yeni vaka oluşturur.

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
      badge.textContent = key.toLowerCase() === "enter" ? "↵" : key.toUpperCase();
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
  setLanguage,
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
  let lastVakaSequenceTime = 0;
  const VAKA_TIMEOUT_MS = 2500;

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

  const isCaseSidebarOpen = () => {
    if (typeof document === "undefined") return false;
    const sidebar = typeof document.getElementById === "function"
      ? document.getElementById("home-case-sidebar")
      : (typeof document.querySelector === "function" ? document.querySelector("#home-case-sidebar") : null);
    return Boolean(sidebar && sidebar.classList && typeof sidebar.classList.contains === "function" && sidebar.classList.contains("is-open"));
  };

  const selectCaseByIndex = (digit) => {
    if (typeof document === "undefined" || !document.querySelector) return false;
    // 1) data-case-index olan vaka kartı
    const targetEl = document.querySelector(`.case-sidebar-card[data-case-index="${digit}"]`)
      || document.querySelector(`#case-sidebar-list .case-sidebar-card:nth-child(${digit})`);
    if (targetEl && typeof targetEl.click === "function") {
      targetEl.click();
      return true;
    }
    return false;
  };

  const triggerNewCase = () => {
    if (typeof document === "undefined" || !document.querySelector) return false;
    const newBtn = document.querySelector('[data-action="new-case-prompt"]');
    if (newBtn && typeof newBtn.click === "function") {
      newBtn.click();
      return true;
    }
    return false;
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

    // Escape basıldığında açık modalları, menüleri veya sihirbazdan geri gitmeyi işle
    if (event.key === "Escape") {
      hideHints();
      lastVakaSequenceTime = 0;
      if (typeof closeNavMenu === "function" && state?.navMenu?.isOpen) {
        closeNavMenu();
        return;
      }
      if (isCaseSidebarOpen()) {
        if (typeof toggleCaseSidebar === "function") toggleCaseSidebar();
        return;
      }
      const backBtn = document.querySelector?.(".workflow-back-btn")
        || document.querySelector?.(".android-back-button")
        || document.querySelector?.('[data-action="android-back"]');
      if (backBtn && typeof backBtn.click === "function") {
        backBtn.click();
        return;
      }
      return;
    }

    // Rakam tuşunu tespit et (Digit1..Digit9, Numpad1..Numpad9 veya doğrudan karakter)
    let digitKey = null;
    if (event.code && event.code.startsWith("Digit")) {
      digitKey = event.code.slice(5);
    } else if (event.code && event.code.startsWith("Numpad") && !isNaN(event.code.slice(6))) {
      digitKey = event.code.slice(6);
    } else if (/^[0-9]$/.test(event.key || "")) {
      digitKey = event.key;
    }

    const rawKey = event.key || "";
    const key = rawKey.toUpperCase();
    const isVakaSequenceActive = (Date.now() - lastVakaSequenceTime) < VAKA_TIMEOUT_MS;

    // 1) VAKA SEÇİMİ VE YENİ VAKA OLUŞTURMA:
    // YALNIZCA kullanıcı önce Shift + C veya Shift + V tuşlayarak vaka sırasını başlattıysa:
    if (isVakaSequenceActive) {
      if (digitKey) {
        if (selectCaseByIndex(digitKey)) {
          lastVakaSequenceTime = 0;
          event.preventDefault();
          event.stopPropagation();
          return;
        }
      }
      if (key === "N") {
        if (triggerNewCase()) {
          lastVakaSequenceTime = 0;
          event.preventDefault();
          event.stopPropagation();
          return;
        }
      }
    }

    // Shift tuşu basılıyken bir tuşa basıldıysa
    if (event.shiftKey) {
      if (!isShiftActive) {
        showHints();
      }

      // Amele Radyal Menü Aç/Kapat (Shift + M)
      if (key === "M") {
        event.preventDefault();
        event.stopPropagation();
        lastVakaSequenceTime = 0;
        if (typeof toggleNavMenu === "function") {
          toggleNavMenu();
        }
        setTimeout(() => {
          if (isShiftActive) {
            syncShortcutBadges();
          }
        }, 35);
        return;
      }

      // Sol Vaka Paneli Aç/Kapat & Sıralı Dinleyici (Shift + V veya Shift + C)
      if (key === "V" || key === "C") {
        event.preventDefault();
        event.stopPropagation();
        lastVakaSequenceTime = Date.now();
        if (typeof toggleCaseSidebar === "function") {
          toggleCaseSidebar();
        }
        setTimeout(() => {
          if (isShiftActive) {
            syncShortcutBadges();
          }
        }, 35);
        return;
      }

      // Yeni Vaka Doğrudan Kısayolu (Shift + N)
      if (key === "N") {
        if (triggerNewCase()) {
          lastVakaSequenceTime = 0;
          event.preventDefault();
          event.stopPropagation();
          return;
        }
      }

      // Sağ Rapor Paneli Aç/Kapat (Shift + R)
      // (Eğer vaka paneli açıksa ve R basıldıysa vaka yenileme öncelikli)
      if (key === "R") {
        event.preventDefault();
        event.stopPropagation();
        if (isCaseSidebarOpen()) {
          const refreshBtn = document.querySelector?.('.case-sidebar-header-btn[data-action="refresh-cases"]');
          if (refreshBtn && typeof refreshBtn.click === "function") {
            refreshBtn.click();
            return;
          }
        }
        if (typeof toggleReportSidebar === "function") {
          toggleReportSidebar();
        }
        return;
      }

      // Dil Değiştirme Kısayolu (Shift + T)
      if (key === "T") {
        event.preventDefault();
        event.stopPropagation();
        if (typeof setLanguage === "function") {
          setLanguage(state?.language === "tr" ? "en" : "tr");
        } else {
          const langBtn = document.querySelector?.('[data-action="set-language"]:not(.is-active)') || document.querySelector?.('[data-action="set-language"]');
          if (langBtn && typeof langBtn.click === "function") {
            langBtn.click();
          }
        }
        return;
      }

      // Güncelleme Kontrolü (Shift + U)
      if (key === "U") {
        const updateBtn = document.querySelector?.('[data-action="check-update"]') || document.querySelector?.('[data-action="about-check-update"]');
        if (updateBtn && typeof updateBtn.click === "function" && !updateBtn.disabled) {
          event.preventDefault();
          event.stopPropagation();
          updateBtn.click();
          return;
        }
      }

      // Güncelleme İndirme (Shift + I - Settings ekranında indirme butonu varsa)
      if (key === "I") {
        const dlBtn = document.querySelector?.('[data-action="download-update"]');
        if (dlBtn && typeof dlBtn.click === "function" && !dlBtn.disabled && dlBtn.style.display !== "none") {
          event.preventDefault();
          event.stopPropagation();
          dlBtn.click();
          return;
        }
      }

      // Workflow / Android Geri Butonu (Shift + B)
      if (key === "B") {
        const backBtn = document.querySelector?.(".workflow-back-btn")
          || document.querySelector?.(".android-back-button")
          || document.querySelector?.('[data-action="android-back"]');
        if (backBtn && typeof backBtn.click === "function") {
          event.preventDefault();
          event.stopPropagation();
          backBtn.click();
          return;
        }
      }

      // Workflow Taramayı Başlat (Shift + S) - Eğer workflow sayfasındaysa taramaya tıkla
      if (key === "S") {
        const scanBtn = document.querySelector?.('.workflow-target-actions [data-action="scan"]');
        if (scanBtn && typeof scanBtn.click === "function" && !scanBtn.disabled) {
          event.preventDefault();
          event.stopPropagation();
          scanBtn.click();
          return;
        }
      }

      // Workflow Edinimi Başlat (Shift + E)
      if (key === "E") {
        const startBtn = document.querySelector?.('.workflow-target-actions [data-action="start"]:not([hidden]):not([disabled])');
        if (startBtn && typeof startBtn.click === "function") {
          event.preventDefault();
          event.stopPropagation();
          startBtn.click();
          return;
        }
      }

      // Workflow Anahtar Onayla (Shift + K)
      if (key === "K") {
        const approveKeyBtn = document.querySelector?.('[data-action="approve-key"]');
        if (approveKeyBtn && typeof approveKeyBtn.click === "function") {
          event.preventDefault();
          event.stopPropagation();
          approveKeyBtn.click();
          return;
        }
      }

      // Workflow Sıfırla veya Durdur (Shift + X)
      if (key === "X") {
        const resetBtn = document.querySelector?.('[data-action="reset-key"]') || document.querySelector?.('[data-action="stop"]');
        if (resetBtn && typeof resetBtn.click === "function") {
          event.preventDefault();
          event.stopPropagation();
          resetBtn.click();
          return;
        }
      }

      // Workflow Bağlan / Form Onayla (Shift + Enter veya Enter)
      if (event.key === "Enter") {
        const connBtn = document.querySelector?.('[data-action="connect"]')
          || document.querySelector?.('[data-docker-action="scan-remote"]')
          || document.querySelector?.('[data-action="android-remote-connect"]');
        if (connBtn && typeof connBtn.click === "function" && !connBtn.disabled) {
          event.preventDefault();
          event.stopPropagation();
          connBtn.click();
          return;
        }
      }

      // Rakam tuşuna basıldıysa (örn: Windows/Linux Araçlarındaki veya Diğer sayfasındaki 1, 2, 3.. kartları)
      if (digitKey && typeof document.querySelector === "function") {
        const targetEl = document.querySelector(`.page [data-shortcut="${digitKey}"]`)
          || document.querySelector(`[data-shortcut="${digitKey}"]`);
        if (targetEl && typeof targetEl.click === "function" && !targetEl.classList.contains("is-disabled") && !targetEl.disabled) {
          event.preventDefault();
          event.stopPropagation();
          targetEl.click();
          return;
        }
      }

      // Sayfa içindeki doğrudan [data-shortcut="KEY"] elemanını kontrol et
      // Sayfa içi butonlar, genel rota geçişlerinden önceliklidir (örn: Ayarlar'da D tema değiştirir, Docker'da S tarama yapar)
      if (typeof document.querySelector === "function") {
        const inPageEl = document.querySelector(`.page [data-shortcut="${key}" i]`)
          || document.querySelector(`.modal [data-shortcut="${key}" i]`)
          || document.querySelector(`[data-shortcut="${key}" i]:not(.nav-item):not(.orbit-btn)`);
        if (inPageEl && typeof inPageEl.click === "function" && !inPageEl.classList.contains("is-disabled") && !inPageEl.disabled) {
          event.preventDefault();
          event.stopPropagation();
          inPageEl.click();
          return;
        }
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

      // Arayüzdeki diğer herhangi bir [data-shortcut="KEY"] elemanını doğrudan tıkla
      if (typeof document.querySelector === "function") {
        const customEl = document.querySelector(`[data-shortcut="${key}" i]`);
        if (customEl && typeof customEl.click === "function" && !customEl.classList.contains("is-disabled") && !customEl.disabled) {
          event.preventDefault();
          event.stopPropagation();
          customEl.click();
          return;
        }
      }
    } else {
      // Shift basılı DEĞİLKEN:
      // Enter tuşu: Workflow veya tarama sayfasındaysa bağlan/tara butonunu çalıştır
      if (event.key === "Enter") {
        const connBtn = document.querySelector?.('[data-action="connect"]')
          || document.querySelector?.('[data-docker-action="scan-remote"]')
          || document.querySelector?.('[data-action="android-remote-connect"]');
        if (connBtn && typeof connBtn.click === "function" && !connBtn.disabled) {
          event.preventDefault();
          event.stopPropagation();
          connBtn.click();
          return;
        }
      }

      // Rakam tuşuna basıldıysa (örn: 1..6 kartları doğrudan açmak için)
      if (digitKey && typeof document.querySelector === "function") {
        const targetEl = document.querySelector(`.page [data-shortcut="${digitKey}"]`)
          || document.querySelector(`[data-shortcut="${digitKey}"]`);
        if (targetEl && typeof targetEl.click === "function" && !targetEl.classList.contains("is-disabled") && !targetEl.disabled) {
          event.preventDefault();
          event.stopPropagation();
          targetEl.click();
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
    lastVakaSequenceTime = 0;
  };

  const handleVisibilityChange = () => {
    if (document.hidden) {
      hideHints();
      lastVakaSequenceTime = 0;
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
