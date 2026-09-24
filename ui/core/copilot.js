// Copilot AI Asistan, model seçici, Amele kural motoru ve yetki yükseltme yönetimi.

import { createApiRequest } from "./api.js";

const apiRequest = createApiRequest({
  backendAvailable: typeof location !== "undefined" && (location.protocol === "http:" || location.protocol === "https:")
});

export function initCopilot(state, render) {
  if (!state.copilot) {
    state.copilot = {
      agents: [],
      selectedAgent: "agy",
      selectedModel: "",
      selectedScope: "all",
      selectedMode: "ask",
      selectedOpt: "balance",
      promptDraft: "",
      messages: [],
      isGenerating: false,
      isExecuting: false,
      elevationModal: null
    };
  }

  // Ajanları ve modelleri backend'den çek
  loadCopilotAgents(state, render);
}

export async function loadCopilotAgents(state, render) {
  try {
    const res = await apiRequest("/api/ai/agents");
    if (res && res.agents && Array.isArray(res.agents)) {
      state.copilot.agents = res.agents;

      // Tercih edilen ilk kurulu ajanı seç: agy > pi > claude > codex > opencode > amele-expert
      const installedOrder = ["agy", "pi", "claude", "codex", "opencode", "amele-expert"];
      let chosen = null;
      for (const id of installedOrder) {
        const found = res.agents.find((a) => a.id === id && (a.installed || a.id === "amele-expert"));
        if (found) {
          chosen = found;
          break;
        }
      }

      if (chosen) {
        state.copilot.selectedAgent = chosen.id;
        if (chosen.models && chosen.models.length > 0) {
          state.copilot.selectedModel = chosen.models[0].id;
        }
      }

      if (render) render();
    }
  } catch (err) {
    console.warn("Yapay zeka ajanları yüklenemedi:", err);
  }
}

export function handleAgentChange(agentId, state, render) {
  if (!state.copilot) return;
  state.copilot.selectedAgent = agentId;
  const agent = state.copilot.agents.find((a) => a.id === agentId);
  if (agent && agent.models && agent.models.length > 0) {
    state.copilot.selectedModel = agent.models[0].id;
  } else {
    state.copilot.selectedModel = "";
  }
  if (render) render();
}

export function handleModelChange(modelId, state) {
  if (!state.copilot) return;
  state.copilot.selectedModel = modelId;
}

export function handleScopeChange(scopeId, state) {
  if (!state.copilot) return;
  state.copilot.selectedScope = scopeId;
}

export async function submitCopilotPrompt(promptText, state, render, showToast, t) {
  if (!promptText || !promptText.trim()) return;
  const cleanPrompt = promptText.trim();

  if (!state.copilot) initCopilot(state, render);

  state.copilot.messages.push({
    id: `msg-${Date.now()}-${Math.random().toString(36).slice(2, 7)}`,
    role: "user",
    content: cleanPrompt,
    timestamp: Date.now()
  });

  state.copilot.promptDraft = "";
  state.copilot.isGenerating = true;
  render();

  try {
    const payload = {
      prompt: cleanPrompt,
      agent: state.copilot.selectedAgent || "amele-expert",
      model: state.copilot.selectedModel || null,
      case_name: state.activeCase?.case_name || "varsayilan_vaka",
      target_scope: state.copilot.selectedScope || "all"
    };

    const res = await apiRequest("/api/ai/chat", {
      method: "POST",
      body: JSON.stringify(payload)
    });

    if (res && res.ok) {
      state.copilot.messages.push({
        id: `msg-${Date.now()}-${Math.random().toString(36).slice(2, 7)}`,
        role: "assistant",
        content: res.response || "",
        agent: res.agent,
        model: res.model,
        suggested_command: res.suggested_command,
        timestamp: Date.now()
      });
    } else {
      state.copilot.messages.push({
        id: `msg-${Date.now()}-${Math.random().toString(36).slice(2, 7)}`,
        role: "assistant",
        content: "⚠️ Ajan sorgusu tamamlanamadı veya bir hata oluştu.",
        timestamp: Date.now()
      });
    }
  } catch (err) {
    state.copilot.messages.push({
      id: `msg-${Date.now()}-${Math.random().toString(36).slice(2, 7)}`,
      role: "assistant",
      content: `❌ Bağlantı hatası: ${err?.message || err}`,
      timestamp: Date.now()
    });
  } finally {
    state.copilot.isGenerating = false;
    render();
  }
}

export async function executeAmeleCommand(cmd, sudoPassword, windowsConfirmed, messageId, state, render, showToast, t) {
  if (!cmd || !cmd.trim()) return;

  state.copilot.isExecuting = true;
  render();

  try {
    const payload = {
      command: cmd.trim(),
      sudo_password: sudoPassword || null,
      windows_confirmed: windowsConfirmed ? true : null
    };

    const res = await apiRequest("/api/ai/execute-command", {
      method: "POST",
      body: JSON.stringify(payload)
    });

    if (res && res.needs_elevation) {
      state.copilot.elevationModal = {
        isOpen: true,
        os: res.os,
        command: cmd.trim(),
        messageId: messageId,
        reason: res.reason
      };
      state.copilot.isExecuting = false;
      render();
      return;
    }

    // Modal açıksa kapat
    state.copilot.elevationModal = null;

    // Mesaja çalıştırma sonucunu iliştir
    const targetMsg = state.copilot.messages.find((m) => m.id === messageId) || state.copilot.messages[state.copilot.messages.length - 1];
    if (targetMsg) {
      targetMsg.execResult = {
        ok: Boolean(res?.ok),
        stdout: res?.stdout || "",
        stderr: res?.stderr || "",
        exit_code: res?.exit_code ?? (res?.ok ? 0 : 1)
      };
    }

    if (res?.ok) {
      showToast?.(t?.("copilot.exitSuccess") || "Komut başarıyla çalıştırıldı.");
    } else {
      showToast?.(t?.("copilot.exitFailed", { code: res?.exit_code ?? -1 }) || "Komut hata ile sonuçlandı.");
    }
  } catch (err) {
    showToast?.(`Hata: ${err?.message || err}`);
  } finally {
    state.copilot.isExecuting = false;
    render();
  }
}

export function getQuickChipPrompt(chipType, isEn) {
  switch (chipType) {
    case "debug":
      return isEn
        ? "Analyze system logs and diagnose forensic tool or permission errors."
        : "Sistem işlem günlüklerini analiz et ve adli araç/yetki hatalarını teşhis et.";
    case "agent":
      return isEn
        ? "Check active case status, integrity hashes, and connected forensic agents."
        : "Aktif vaka durumunu, bütünlük hashlerini ve bağlı adli ajanları denetle.";
    case "issue":
      return isEn
        ? "Generate a digital forensics triage checklist and evidence acquisition plan."
        : "Dijital delil triyaj kontrol listesi ve adli edinim eylem planı oluştur.";
    case "code":
      return isEn
        ? "Generate a ready-to-run Amele CLI command for physical RAM and disk acquisition."
        : "Fiziksel RAM ve blok disk edinimi için çalıştırılabilir Amele CLI komutu üret.";
    case "git":
      return isEn
        ? "Verify SHA-256 chain of custody and forensic evidence hash integrity."
        : "Delil zinciri bütünlüğü ve SHA-256 adli hash doğrulama komutlarını listele.";
    case "pr":
      return isEn
        ? "Package the active case into an encrypted forensic archive bundle (.tar.gz)."
        : "Aktif vakayı şifreli adli arşiv paketi (.tar.gz) olarak mühürle.";
    default:
      return "";
  }
}
