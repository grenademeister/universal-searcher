import { useEffect, useMemo, useState } from "react";
import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";
import { invoke } from "@tauri-apps/api/core";
import { PhysicalSize } from "@tauri-apps/api/dpi";
import { getCurrentWindow } from "@tauri-apps/api/window";
import "./App.css";

const PROVIDERS = ["wikipedia", "gemini", "openai"];
const MODEL_OPTIONS = {
  wikipedia: ["kiwix-wikipedia"],
  gemini: ["gemini-3-pro-preview", "gemini-2.5-pro", "gemini-2.5-flash", "gemini-2.5-flash-lite"],
  openai: ["gpt-5.1", "gpt-5-mini", "gpt-5-nano"],
};
const DEFAULT_MODELS = {
  wikipedia: "kiwix-wikipedia",
  gemini: "gemini-2.5-flash",
  openai: "gpt-5-mini",
};
const DEFAULT_APPEARANCE = {
  background_color: "transparent",
  font_color: "#e9ecf1",
};

const initialModelIndices = Object.fromEntries(
  Object.entries(MODEL_OPTIONS).map(([prov, options]) => {
    const target = DEFAULT_MODELS[prov];
    const idx = options.indexOf(target);
    return [prov, idx >= 0 ? idx : 0];
  }),
);

const markdownComponents = {
  a: ({ node, ...props }) => (
    <a {...props} target="_blank" rel="noreferrer" />
  ),
};

function App() {
  const [overlay, setOverlay] = useState({
    text: "Loading...",
    model: "",
    provider: PROVIDERS[0],
    query: "",
  });
  const [cache, setCache] = useState({});
  const [modelIndices, setModelIndices] = useState(initialModelIndices);
  const [error, setError] = useState("");
  const [loading, setLoading] = useState(true);
  const [provider, setProvider] = useState(PROVIDERS[0]);

  const getModelForProvider = (prov, indexOverride) => {
    const options = MODEL_OPTIONS[prov] || [];
    if (options.length === 0) return "";
    const idx = ((indexOverride ?? 0) % options.length + options.length) % options.length;
    return options[idx];
  };

  const currentModel = useMemo(
    () => getModelForProvider(provider, modelIndices[provider] ?? 0),
    [provider, modelIndices],
  );

  useEffect(() => {
    fetchOverlay(provider, currentModel);
  }, [provider, currentModel]);

  useEffect(() => {
    async function hydrateConfig() {
      try {
        const cfg = await invoke("load_config");
        applyAppearance({
          background_color:
            cfg?.appearance?.background_color || DEFAULT_APPEARANCE.background_color,
          font_color: cfg?.appearance?.font_color || DEFAULT_APPEARANCE.font_color,
        });
        await applyWindowSize(cfg?.window);
      } catch (err) {
        console.warn("Failed to load config; using defaults.", err);
        applyAppearance(DEFAULT_APPEARANCE);
      }
    }

    hydrateConfig();
  }, []);

  useEffect(() => {
    const onError = (event) => {
      setError(
        event?.error?.message ||
        event?.message ||
        "Unexpected error. Check console logs.",
      );
    };
    const onRejection = (event) => {
      setError(
        event?.reason?.message ||
        event?.reason?.toString() ||
        "Unexpected promise rejection.",
      );
    };
    window.addEventListener("error", onError);
    window.addEventListener("unhandledrejection", onRejection);
    return () => {
      window.removeEventListener("error", onError);
      window.removeEventListener("unhandledrejection", onRejection);
    };
  }, []);

  useEffect(() => {
    const onKey = (event) => {
      if (event.key === "Tab") {
        event.preventDefault();
        setProvider((current) => {
          const index = PROVIDERS.indexOf(current);
          if (index === -1) {
            return PROVIDERS[0];
          }
          const nextIndex = (index + 1) % PROVIDERS.length;
          return PROVIDERS[nextIndex];
        });
        return;
      }

      if (event.key === "ArrowUp" || event.key === "ArrowDown") {
        event.preventDefault();
        setModelIndices((current) => {
          const options = MODEL_OPTIONS[provider] || [];
          if (options.length === 0) return current;
          const delta = event.key === "ArrowUp" ? -1 : 1;
          const nextIndex =
            ((current[provider] ?? 0) + delta + options.length) % options.length;
          return { ...current, [provider]: nextIndex };
        });
        return;
      }

      if (event.code === "Space" || event.key === " ") {
        event.preventDefault();
        invoke("shutdown").catch(() => { });
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [provider]);

  const nextProvider = (current) => {
    const index = PROVIDERS.indexOf(current);
    if (index === -1) return PROVIDERS[0];
    return PROVIDERS[(index + 1) % PROVIDERS.length];
  };

  const makeCacheKey = (prov, modelName) => `${prov}:${modelName}`;

  const applyAppearance = (appearanceConfig) => {
    const bg = appearanceConfig?.background_color || DEFAULT_APPEARANCE.background_color;
    const fg = appearanceConfig?.font_color || DEFAULT_APPEARANCE.font_color;
    const root = document.documentElement;
    root.style.setProperty("--overlay-bg", bg);
    root.style.setProperty("--overlay-text", fg);
  };

  const applyWindowSize = async (windowConfig) => {
    const width = Number(windowConfig?.width);
    const height = Number(windowConfig?.height);
    if (!width || !height) return;
    try {
      const current = getCurrentWindow();
      await current.setSize(new PhysicalSize(width, height));
    } catch (err) {
      console.warn("Unable to resize window from config.", err);
    }
  };

  async function prefetchOverlay(targetProvider, targetModel) {
    if (!targetProvider || !targetModel) return;
    const cacheKey = makeCacheKey(targetProvider, targetModel);
    if (cache[cacheKey]) return;
    try {
      const result = await invoke("generate_overlay", {
        provider: targetProvider,
        model: targetModel,
      });
      const formatted = {
        text: result?.text ?? "",
        model: result?.model ?? "",
        provider: result?.provider ?? targetProvider,
        query: result?.query ?? "",
      };
      setCache((prev) => ({ ...prev, [cacheKey]: formatted }));
    } catch {
      // best-effort prefetch; ignore errors
    }
  }

  async function fetchOverlay(targetProvider, targetModel) {
    setLoading(true);
    setError("");
    setOverlay((prev) => ({
      ...prev,
      text: "Loading...",
      provider: targetProvider,
      model: targetModel,
      query: prev.query,
    }));

    const cacheKey = makeCacheKey(targetProvider, targetModel);
    const cached = cache[cacheKey];
    if (cached && cached.query) {
      setOverlay(cached);
      setLoading(false);
      const nextProv = nextProvider(targetProvider);
      prefetchOverlay(
        nextProv,
        getModelForProvider(nextProv, modelIndices[nextProv] ?? 0),
      );
      return;
    }

    try {
      const result = await invoke("generate_overlay", {
        provider: targetProvider,
        model: targetModel,
      });
      const formatted = {
        text: result?.text ?? "",
        model: result?.model ?? "",
        provider: result?.provider ?? targetProvider,
        query: result?.query ?? "",
      };
      setOverlay(formatted);
      setCache((prev) => ({ ...prev, [cacheKey]: formatted }));
      const nextProv = nextProvider(formatted.provider);
      prefetchOverlay(
        nextProv,
        getModelForProvider(nextProv, modelIndices[nextProv] ?? 0),
      );
    } catch (err) {
      setError(err?.toString() ?? "Failed to generate overlay");
    } finally {
      setLoading(false);
    }
  }

  return (
    <main className="overlay">
      <section className="panel">
        <p className="overlay-meta">
          Model: {loading
            ? "Loading…"
            : overlay.model || overlay.provider || "unknown"}
        </p>
        <p className="overlay-query">
          <strong>Query:</strong>{" "}
          {loading
            ? "Loading…"
            : overlay.query.length > 50
              ? `${overlay.query.slice(0, 47)}...`
              : overlay.query || "(empty)"}
        </p>
        <div className={`overlay-text ${error ? "error" : ""}`}>
          {loading && <p className="overlay-status">Loading…</p>}
          {!loading && error && <p className="overlay-status">{error}</p>}
          {!loading && !error && (
            <ReactMarkdown
              remarkPlugins={[remarkGfm]}
              components={markdownComponents}
            >
              {overlay.text || ""}
            </ReactMarkdown>
          )}
        </div>
      </section>
    </main>
  );
}

export default App;
