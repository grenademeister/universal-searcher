import { useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";

function App() {
  const [text, setText] = useState("Loading...");
  const [error, setError] = useState("");
  const [loading, setLoading] = useState(true);
  const [provider, setProvider] = useState("openai");
  const panelRef = useRef(null);

  useEffect(() => {
    run(provider);
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
          void run("gemini");
        } else {
          setProvider("gemini");
        }
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [provider]);

  async function run(targetProvider) {
    setLoading(true);
    setError("");
    try {
      const result = await invoke("run_overlay_cli", {
        provider: targetProvider,
      });
      setText(result ?? "");
    } catch (err) {
      setError(err?.toString() ?? "Failed to run overlay-cli");
    } finally {
      setLoading(false);
    }
  }

  return (
    <main className="overlay">
      <section className="panel" ref={panelRef}>
        <p className={`overlay-text ${error ? "error" : ""}`}>
          {loading ? "Loading…" : error || text}
        </p>
      </section>
    </main>
  );
}

export default App;
