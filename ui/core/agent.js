// Yapay Zeka Adli Ajan, model seçici, Amele kural motoru ve yetki yükseltme yönetimi.

import { createApiRequest } from "./api.js";

const apiRequest = createApiRequest({
  backendAvailable: typeof location !== "undefined" && (location.protocol === "http:" || location.protocol === "https:")
});

export const DEFAULT_AGENTS = [
  {
    id: "agy",
    name: "Antigravity (AGY)",
    installed: true,
    models: [
      { id: "gemini-3.8-flash-high", name: "Gemini 3.8 Flash (High)", description: "En yeni yüksek hızlı akıl yürütme modeli" },
      { id: "gemini-3.1-pro-high", name: "Gemini 3.1 Pro (High)", description: "Karmaşık adli bilişim analizi ve derin akıl yürütme" },
      { id: "gemini-3.7-flash-high", name: "Gemini 3.7 Flash", description: "Hızlı genel adli bilişim sorguları" },
      { id: "claude-sonnet-4-6", name: "Claude Sonnet 4.6 (Thinking)", description: "Gelişmiş analitik akıl yürütme" },
      { id: "claude-opus-4-6-thinking", name: "Claude Opus 4.6 (Thinking)", description: "Üst seviye stratejik analiz modeli" }
    ]
  },
  {
    id: "claude",
    name: "Claude Code",
    installed: true,
    models: [
      { id: "claude-3-7-sonnet", name: "Claude 3.7 Sonnet", description: "Hibrit akıl yürütme ve adli kodlama" },
      { id: "claude-3-5-sonnet", name: "Claude 3.5 Sonnet v2", description: "Yüksek doğrulukta komut üretimi" },
      { id: "claude-3-5-haiku", name: "Claude 3.5 Haiku", description: "Hafif ve ultra hızlı yanıt süresi" }
    ]
  },
  {
    id: "codex",
    name: "Codex / OpenAI",
    installed: true,
    models: [
      { id: "gpt-4o", name: "GPT-4o", description: "En yetenekli amiral gemisi model" },
      { id: "o3-mini", name: "o3-mini", description: "Gelişmiş mantık ve akıl yürütme" },
      { id: "gpt-4o-mini", name: "GPT-4o Mini", description: "Hızlı ve ekonomik model" }
    ]
  },
  {
    id: "pi",
    name: "Pi Coding Agent",
    installed: true,
    models: [
      { id: "claude-sonnet", name: "Claude Sonnet (Pi)", description: "Dengeli ve güçlü adli analiz" },
      { id: "gpt-4o", name: "GPT-4o (Pi)", description: "Çok modlu ve kapsamlı yanıtlar" },
      { id: "claude-haiku", name: "Claude Haiku (Pi)", description: "Ultra hızlı yanıt süresi" },
      { id: "deepseek-r1", name: "DeepSeek R1 (Pi)", description: "Yerel ve derin akıl yürütme" }
    ]
  },
  {
    id: "opencode",
    name: "OpenCode",
    installed: true,
    models: [
      { id: "opencode/big-pickle", name: "Big Pickle", description: "OpenCode genel amaçlı model" },
      { id: "opencode/ling-3.0-flash-fin-free", name: "Ling 3.0 Flash", description: "Hızlı ve ücretsiz model" },
      { id: "opencode/mimo-v2.6-flash-free", name: "Mimo v2.6 Flash", description: "Hafif analiz modeli" }
    ]
  }
];

export function initAgent(state, render) {
  if (!state.agent) {
    state.agent = {
      agents: [...DEFAULT_AGENTS],
      selectedAgent: "agy",
      selectedModel: "gemini-3.8-flash-high",
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
  state.copilot = state.agent; // Geriye dönük uyumluluk

  // Ajanları ve modelleri backend'den çek
  loadAgents(state, render);
}

export async function loadAgents(state, render) {
  try {
    const res = await apiRequest("/api/ai/agents");
    if (res && res.agents && Array.isArray(res.agents)) {
      if (!state.agent) initAgent(state, render);
      state.agent.agents = res.agents;

      // Sistemde kurulu olan ilk ajanı seç
      const installedAgent = res.agents.find((a) => a.installed);
      const chosen = installedAgent || res.agents[0];

      if (chosen) {
        state.agent.selectedAgent = chosen.id;
        if (chosen.models && chosen.models.length > 0) {
          state.agent.selectedModel = chosen.models[0].id;
        }
      }

      if (render) render();
    }
  } catch (err) {
    console.warn("Yapay zeka ajanları yüklenemedi:", err);
  }
}

export function handleAgentChange(agentId, state, render) {
  if (!state.agent) return;
  const agents = (state.agent.agents && state.agent.agents.length > 0) ? state.agent.agents : DEFAULT_AGENTS;
  const targetAgent = agents.find((a) => a.id === agentId);
  
  // Kurulu olmayan agent seçilemez
  if (targetAgent && targetAgent.installed === false) {
    return;
  }

  state.agent.selectedAgent = agentId;
  if (targetAgent && targetAgent.models && targetAgent.models.length > 0) {
    state.agent.selectedModel = targetAgent.models[0].id;
  } else {
    state.agent.selectedModel = "";
  }
  if (render) render();
}

export function handleModelChange(modelId, state) {
  if (!state.agent) return;
  state.agent.selectedModel = modelId;
}

export function handleScopeChange(scopeId, state) {
  if (!state.agent) return;
  state.agent.selectedScope = scopeId;
}

export async function submitAgentPrompt(promptText, state, render, showToast, t) {
  if (!promptText || !promptText.trim()) return;
  const cleanPrompt = promptText.trim();

  if (!state.agent) initAgent(state, render);

  state.agent.messages.push({
    id: `msg-${Date.now()}-${Math.random().toString(36).slice(2, 7)}`,
    role: "user",
    content: cleanPrompt,
    timestamp: Date.now()
  });

  state.agent.promptDraft = "";
  state.agent.isGenerating = true;
  render();

  try {
    const payload = {
      prompt: cleanPrompt,
      agent: state.agent.selectedAgent || "amele-expert",
      model: state.agent.selectedModel || null,
      case_name: state.activeCase?.case_name || "varsayilan_vaka",
      target_scope: state.agent.selectedScope || "all"
    };

    const res = await apiRequest("/api/ai/chat", {
      method: "POST",
      body: JSON.stringify(payload)
    });

    if (res && res.ok) {
      state.agent.messages.push({
        id: `msg-${Date.now()}-${Math.random().toString(36).slice(2, 7)}`,
        role: "assistant",
        content: res.response || "",
        agent: res.agent,
        model: res.model,
        suggested_command: res.suggested_command,
        timestamp: Date.now()
      });
    } else {
      state.agent.messages.push({
        id: `msg-${Date.now()}-${Math.random().toString(36).slice(2, 7)}`,
        role: "assistant",
        content: "⚠️ Ajan sorgusu tamamlanamadı veya bir hata oluştu.",
        timestamp: Date.now()
      });
    }
  } catch (err) {
    state.agent.messages.push({
      id: `msg-${Date.now()}-${Math.random().toString(36).slice(2, 7)}`,
      role: "assistant",
      content: `❌ Bağlantı hatası: ${err?.message || err}`,
      timestamp: Date.now()
    });
  } finally {
    state.agent.isGenerating = false;
    render();
  }
}

export async function executeAmeleCommand(cmd, sudoPassword, windowsConfirmed, messageId, state, render, showToast, t) {
  if (!cmd || !cmd.trim()) return;

  if (!state.agent) initAgent(state, render);
  state.agent.isExecuting = true;
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
      state.agent.elevationModal = {
        isOpen: true,
        os: res.os,
        command: cmd.trim(),
        messageId: messageId,
        reason: res.reason
      };
      state.agent.isExecuting = false;
      render();
      return;
    }

    // Modal açıksa kapat
    state.agent.elevationModal = null;

    // Mesaja çalıştırma sonucunu iliştir
    const targetMsg = state.agent.messages.find((m) => m.id === messageId) || state.agent.messages[state.agent.messages.length - 1];
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
    state.agent.isExecuting = false;
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
