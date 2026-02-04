import { useState } from 'react'

interface CollectionManagerProps {
    apiKey: string
    collections: string[]
    onRefresh: () => void
    onSelectCollection: (name: string) => void
}

const COLLECTION_PRESETS = [
    { label: 'Hyperbolic 16', dimension: 16, metric: 'poincare' },
    { label: 'Hyperbolic 32', dimension: 32, metric: 'poincare' },
    { label: 'Hyperbolic 64', dimension: 64, metric: 'poincare' },
    { label: 'Hyperbolic 128', dimension: 128, metric: 'poincare' },
    { label: 'Euclidean 1024', dimension: 1024, metric: 'euclidean' },
    { label: 'Euclidean 1536', dimension: 1536, metric: 'euclidean' },
    { label: 'Euclidean 2048', dimension: 2048, metric: 'euclidean' },
]

export default function CollectionManager({ apiKey, collections, onRefresh, onSelectCollection }: CollectionManagerProps) {
    const [newColName, setNewColName] = useState('')
    const [selectedPreset, setSelectedPreset] = useState(0)
    const [stats, setStats] = useState<Record<string, any>>({})

    const createCollection = async () => {
        if (!newColName.trim()) return

        const preset = COLLECTION_PRESETS[selectedPreset]
        try {
            await fetch('/api/collections', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                    'x-api-key': apiKey
                },
                body: JSON.stringify({
                    name: newColName,
                    dimension: preset.dimension,
                    metric: preset.metric
                })
            })
            setNewColName('')
            onRefresh()
        } catch (err) {
            console.error(err)
        }
    }

    const deleteCollection = async (name: string) => {
        if (!confirm(`Delete collection "${name}"?`)) return

        try {
            await fetch(`/api/collections/${name}`, {
                method: 'DELETE',
                headers: { 'x-api-key': apiKey }
            })
            onRefresh()
        } catch (err) {
            console.error(err)
        }
    }

    const fetchStats = async (name: string) => {
        try {
            const res = await fetch(`/api/collections/${name}/stats`, {
                headers: { 'x-api-key': apiKey }
            })
            if (res.ok) {
                const data = await res.json()
                setStats(prev => ({ ...prev, [name]: data }))
            }
        } catch (err) {
            console.error(err)
        }
    }

    return (
        <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '20px' }}>
            {/* Create Collection */}
            <div className="card">
                <h2>Create Collection</h2>
                <div style={{ display: 'flex', flexDirection: 'column', gap: '10px' }}>
                    <input
                        placeholder="Collection Name"
                        value={newColName}
                        onChange={e => setNewColName(e.target.value)}
                        style={{ padding: '10px', fontSize: '14px' }}
                    />
                    <select
                        value={selectedPreset}
                        onChange={e => setSelectedPreset(parseInt(e.target.value))}
                        style={{ padding: '10px', fontSize: '14px' }}
                    >
                        {COLLECTION_PRESETS.map((preset, idx) => (
                            <option key={idx} value={idx}>
                                {preset.label} ({preset.dimension}D, {preset.metric})
                            </option>
                        ))}
                    </select>
                    <button onClick={createCollection} style={{ padding: '10px' }}>Create</button>
                </div>
            </div>

            {/* Collections List */}
            <div className="card">
                <h2>Active Collections ({collections.length})</h2>
                <div style={{ maxHeight: '400px', overflowY: 'auto' }}>
                    {collections.map(c => (
                        <div
                            key={c}
                            style={{
                                margin: '10px 0',
                                border: '1px solid #333',
                                padding: '15px',
                                borderRadius: '8px',
                                cursor: 'pointer'
                            }}
                            onClick={() => {
                                onSelectCollection(c)
                                fetchStats(c)
                            }}
                        >
                            <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
                                <div>
                                    <strong>{c}</strong>
                                    {stats[c] && (
                                        <div style={{ fontSize: '12px', color: '#888', marginTop: '5px' }}>
                                            Vectors: {stats[c].count} | Dim: {stats[c].dimension || 'N/A'} | Metric: {stats[c].metric}
                                        </div>
                                    )}
                                </div>
                                <button
                                    onClick={(e) => {
                                        e.stopPropagation()
                                        deleteCollection(c)
                                    }}
                                    style={{ backgroundColor: '#ff4444', padding: '5px 10px' }}
                                >
                                    Delete
                                </button>
                            </div>
                        </div>
                    ))}
                    {collections.length === 0 && <p style={{ color: '#666' }}>No collections found.</p>}
                </div>
            </div>
        </div>
    )
}
