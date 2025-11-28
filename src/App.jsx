import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";

const PROVIDERS = ["wikipedia", "gemini", "openai"];

function App() {
  const [overlay, setOverlay] = useState({
    text: "Loading...",
    model: "",
    provider: PROVIDERS[0],
  });
  const [cache, setCache] = useState({});
  const [error, setError] = useState("");
  const [loading, setLoading] = useState(true);
  const [provider, setProvider] = useState(PROVIDERS[0]);

  useEffect(() => {
    fetchOverlay(provider);
  }, [provider]);

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
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, []);

  const nextProvider = (current) => {
    const index = PROVIDERS.indexOf(current);
    if (index === -1) return PROVIDERS[0];
    return PROVIDERS[(index + 1) % PROVIDERS.length];
  };

  async function prefetchOverlay(targetProvider) {
    if (cache[targetProvider]) return;
    try {
      const result = await invoke("generate_overlay", {
        provider: targetProvider,
      });
      const formatted = {
        text: result?.text ?? "",
        model: result?.model ?? "",
        provider: result?.provider ?? targetProvider,
      };
      setCache((prev) => ({ ...prev, [formatted.provider]: formatted }));
    } catch {
      // best-effort prefetch; ignore errors
    }
  }

  async function fetchOverlay(targetProvider) {
    setLoading(true);
    setError("");

    const cached = cache[targetProvider];
    if (cached) {
      setOverlay(cached);
      setLoading(false);
      prefetchOverlay(nextProvider(targetProvider));
      return;
    }

    try {
      const result = await invoke("generate_overlay", {
        provider: targetProvider,
      });
      const formatted = {
        text: result?.text ?? "",
        model: result?.model ?? "",
        provider: result?.provider ?? targetProvider,
      };
      setOverlay(formatted);
      setCache((prev) => ({ ...prev, [formatted.provider]: formatted }));
      prefetchOverlay(nextProvider(formatted.provider));
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
        <p className={`overlay-text ${error ? "error" : ""}`}>
          {loading ? "Loading…" : error || overlay.text}
        </p>
      </section>
    </main>
  );
}

export default App;
