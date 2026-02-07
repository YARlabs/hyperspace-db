import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/tauri";
import { open } from "@tauri-apps/api/dialog";

function App() {
    const [stats, setStats] = useState<[number, number]>([0, 0]);
    const [status, setStatus] = useState("Ready");

    useEffect(() => {
        updateStats();
    }, []);

    async function updateStats() {
        try {
            const s = await invoke<[number, number]>("get_stats");
            setStats(s);
        } catch (e) { console.error(e); }
    }

    async function handleIngest() {
        try {
            const selected = await open({
                multiple: false,
                filters: [{ name: 'PDF', extensions: ['pdf'] }]
            });

            if (typeof selected === "string") {
                setStatus("Ingesting " + selected + "...");
                try {
                    await invoke("ingest_pdf", { path: selected });
                    setStatus("Ingested!");
                    updateStats();
                } catch (e) {
                    setStatus("Error: " + String(e));
                }
            }
        } catch (e) {
            setStatus("Dialog Error: " + String(e));
        }
    }

    return (
        <div style={{ padding: 20, fontFamily: 'sans-serif', background: '#222', color: '#fff', minHeight: '100vh' }}>
            <h1>HiveMind 🧠</h1>
            <div style={{ display: 'flex', gap: 20, marginBottom: 20 }}>
                <div style={{ background: '#333', padding: 20, borderRadius: 8 }}>
                    <h3>Stats</h3>
                    <p>Vectors: {stats[0]}</p>
                    <p>Storage: {(stats[1] / 1024 / 1024).toFixed(2)} MB</p>
                </div>
            </div>

            <button
                onClick={handleIngest}
                style={{ padding: '10px 20px', background: '#646cff', color: 'white', border: 'none', borderRadius: 4, cursor: 'pointer' }}
            >
                📂 Ingest PDF
            </button>

            <div style={{ marginTop: 20, color: '#aaa' }}>Status: {status}</div>
        </div>
    );
}

export default App;
