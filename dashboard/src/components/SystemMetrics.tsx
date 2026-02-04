import { useState, useEffect } from 'react'

interface SystemMetricsProps {
    apiKey: string
}

interface Stats {
    total_collections: number
    total_vectors: number
    total_memory_mb: number
    qps: number
}

export default function SystemMetrics({ apiKey }: SystemMetricsProps) {
    const [stats, setStats] = useState<Stats | null>(null)
    const [history, setHistory] = useState<Stats[]>([])

    useEffect(() => {
        const fetchStats = async () => {
            try {
                // For now, we'll aggregate from collections
                // In production, use the monitor stream or a dedicated endpoint
                const res = await fetch('/api/collections', {
                    headers: { 'x-api-key': apiKey }
                })

                if (res.ok) {
                    const collections = await res.json()

                    // Fetch stats for each collection
                    let totalVectors = 0
                    for (const col of collections) {
                        const statsRes = await fetch(`/api/collections/${col}/stats`, {
                            headers: { 'x-api-key': apiKey }
                        })
                        if (statsRes.ok) {
                            const colStats = await statsRes.json()
                            totalVectors += colStats.count || 0
                        }
                    }

                    const currentStats: Stats = {
                        total_collections: collections.length,
                        total_vectors: totalVectors,
                        total_memory_mb: 0, // TODO: implement
                        qps: 0 // TODO: implement
                    }

                    setStats(currentStats)
                    setHistory(prev => [...prev.slice(-59), currentStats])
                }
            } catch (err) {
                console.error(err)
            }
        }

        fetchStats()
        const interval = setInterval(fetchStats, 2000)
        return () => clearInterval(interval)
    }, [apiKey])

    if (!stats) return <div>Loading...</div>

    return (
        <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(250px, 1fr))', gap: '20px' }}>
            {/* Metrics Cards */}
            <div className="card" style={{ background: 'linear-gradient(135deg, #1a1a2e 0%, #16213e 100%)' }}>
                <h3 style={{ margin: '0 0 10px 0', color: '#00ffff' }}>Collections</h3>
                <div style={{ fontSize: '48px', fontWeight: 'bold' }}>{stats.total_collections}</div>
                <div style={{ fontSize: '12px', color: '#888', marginTop: '5px' }}>Active collections</div>
            </div>

            <div className="card" style={{ background: 'linear-gradient(135deg, #1a1a2e 0%, #16213e 100%)' }}>
                <h3 style={{ margin: '0 0 10px 0', color: '#00ffff' }}>Total Vectors</h3>
                <div style={{ fontSize: '48px', fontWeight: 'bold' }}>{stats.total_vectors.toLocaleString()}</div>
                <div style={{ fontSize: '12px', color: '#888', marginTop: '5px' }}>Indexed vectors</div>
            </div>

            <div className="card" style={{ background: 'linear-gradient(135deg, #1a1a2e 0%, #16213e 100%)' }}>
                <h3 style={{ margin: '0 0 10px 0', color: '#00ffff' }}>Memory Usage</h3>
                <div style={{ fontSize: '48px', fontWeight: 'bold' }}>{stats.total_memory_mb.toFixed(1)}</div>
                <div style={{ fontSize: '12px', color: '#888', marginTop: '5px' }}>MB allocated</div>
            </div>

            <div className="card" style={{ background: 'linear-gradient(135deg, #1a1a2e 0%, #16213e 100%)' }}>
                <h3 style={{ margin: '0 0 10px 0', color: '#00ffff' }}>QPS</h3>
                <div style={{ fontSize: '48px', fontWeight: 'bold' }}>{stats.qps.toFixed(2)}</div>
                <div style={{ fontSize: '12px', color: '#888', marginTop: '5px' }}>Queries per second</div>
            </div>

            {/* History Chart */}
            <div className="card" style={{ gridColumn: '1 / -1' }}>
                <h3>Vector Count History</h3>
                <div style={{ height: '200px', position: 'relative', marginTop: '20px' }}>
                    <svg width="100%" height="100%" style={{ border: '1px solid #333', borderRadius: '4px' }}>
                        {/* Grid lines */}
                        {[0, 1, 2, 3, 4].map(i => (
                            <line
                                key={i}
                                x1="0"
                                y1={`${i * 25}%`}
                                x2="100%"
                                y2={`${i * 25}%`}
                                stroke="#333"
                                strokeWidth="1"
                            />
                        ))}

                        {/* Data line */}
                        {history.length > 1 && (
                            <polyline
                                points={history.map((s, i) => {
                                    const x = (i / (history.length - 1)) * 100
                                    const maxVectors = Math.max(...history.map(h => h.total_vectors), 1)
                                    const y = 100 - (s.total_vectors / maxVectors) * 90
                                    return `${x}%,${y}%`
                                }).join(' ')}
                                fill="none"
                                stroke="#00ffff"
                                strokeWidth="2"
                            />
                        )}
                    </svg>
                </div>
            </div>

            {/* System Info */}
            <div className="card" style={{ gridColumn: '1 / -1' }}>
                <h3>System Information</h3>
                <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '10px', marginTop: '15px' }}>
                    <div>
                        <strong>Version:</strong> v1.1.0
                    </div>
                    <div>
                        <strong>Build:</strong> Multi-Tenant
                    </div>
                    <div>
                        <strong>Architecture:</strong> HNSW + Poincaré
                    </div>
                    <div>
                        <strong>Storage:</strong> Memory-mapped
                    </div>
                </div>
            </div>
        </div>
    )
}
