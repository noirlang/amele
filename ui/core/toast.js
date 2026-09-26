// ekranın köşesinde çıkan toast bildirim kutuları.

import { normalizeErrorMessage } from "./errors.js";

let toastTimer = null;

export function showToast(message, type = "success") {
  let toast = document.querySelector(".toast");
  if (!toast) {
    toast = document.createElement("div");
    toast.className = "toast";
    toast.onclick = () => {
      toast.classList.remove("visible");
      window.clearTimeout(toastTimer);
    };
    document.body.appendChild(toast);
  } else if (!toast.onclick) {
    toast.onclick = () => {
      toast.classList.remove("visible");
      window.clearTimeout(toastTimer);
    };
  }

  const displayMessage = type === "error" ? normalizeErrorMessage(message) : String(message ?? "");
  toast.textContent = displayMessage;
  toast.title = displayMessage;
  toast.dataset.type = type;
  toast.classList.add("visible");

  window.clearTimeout(toastTimer);
  toastTimer = window.setTimeout(
    () => toast.classList.remove("visible"),
    type === "error" ? Math.min(18000, 9000 + displayMessage.length * 18) : 3200
  );
}
