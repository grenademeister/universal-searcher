import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";

function App() {
  const [overlay, setOverlay] = useState({
    text: "Loading...",
    model: "",
    provider: "openai",
  });
  const [error, setError] = useState("");
  const [loading, setLoading] = useState(true);
  const [provider, setProvider] = useState("openai");

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
        if (provider === "gemini") {
          void fetchOverlay("gemini");
        } else {
          setProvider("gemini");
        }
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [provider]);

  async function fetchOverlay(targetProvider) {
    setLoading(true);
    setError("");
    try {
      const result = await invoke("generate_overlay", {
        provider: targetProvider,
      });
      setOverlay({
        text: result?.text ?? "",
        model: result?.model ?? "",
        provider: result?.provider ?? targetProvider,
      });
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
          Model: {overlay.model || overlay.provider || "unknown"}
        </p>
        <p className={`overlay-text ${error ? "error" : ""}`}>
          {loading ? "Loading…" : error || overlay.text}
        </p>
      </section>
    </main>
  );
}

export default App;
