import { useState, useEffect } from 'react'
import CollectionManager from './CollectionManager'
import PoincareVisualizer from './PoincareVisualizer'
import SystemMetrics from './SystemMetrics'

interface DashboardProps {
    apiKey: string
    onLogout: () => void
    logo: string
}

export default function Dashboard({ apiKey, onLogout, logo }: DashboardProps) {
    const [collections, setCollections] = useState<string[]>([])
    const [selectedCollection, setSelectedCollection] = useState<string | null>(null)
    const [activeTab, setActiveTab] = useState<'collections' | 'visualizer' | 'metrics'>('collections')

    const fetchCollections = async () => {
        try {
            const res = await fetch('/api/collections', {
                headers: { 'x-api-key': apiKey }
            })
            if (res.ok) {
                const data = await res.json()
                setCollections(data)
            }
        } catch (err) {
            console.error(err)
        }
    }

    useEffect(() => {
        fetchCollections()
        const interval = setInterval(fetchCollections, 3000)
        return () => clearInterval(interval)
    }, [apiKey])

    return (
        <div style={{ padding: '20px', maxWidth: '1400px', margin: '0 auto' }}>
            {/* Header */}
            <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '30px' }}>
                <div style={{ display: 'flex', alignItems: 'center', gap: '15px' }}>
                    <img src={logo} alt="HyperspaceDB" style={{ width: '50px' }} />
                    <h1 style={{ margin: 0 }}>HyperspaceDB Dashboard</h1>
                </div>
                <button onClick={onLogout} style={{ padding: '8px 16px' }}>Logout</button>
            </div>

            {/* Tabs */}
            <div style={{ display: 'flex', gap: '10px', marginBottom: '20px', borderBottom: '2px solid #333' }}>
                <button
                    onClick={() => setActiveTab('collections')}
                    style={{
                        padding: '10px 20px',
                        background: activeTab === 'collections' ? '#00ffff' : 'transparent',
                        color: activeTab === 'collections' ? '#000' : '#fff',
                        border: 'none',
                        cursor: 'pointer'
                    }}
                >
                    Collections
                </button>
                <button
                    onClick={() => setActiveTab('visualizer')}
                    style={{
                        padding: '10px 20px',
                        background: activeTab === 'visualizer' ? '#00ffff' : 'transparent',
                        color: activeTab === 'visualizer' ? '#000' : '#fff',
                        border: 'none',
                        cursor: 'pointer'
                    }}
                >
                    Poincaré Visualizer
                </button>
                <button
                    onClick={() => setActiveTab('metrics')}
                    style={{
                        padding: '10px 20px',
                        background: activeTab === 'metrics' ? '#00ffff' : 'transparent',
                        color: activeTab === 'metrics' ? '#000' : '#fff',
                        border: 'none',
                        cursor: 'pointer'
                    }}
                >
                    System Metrics
                </button>
            </div>

            {/* Content */}
            {activeTab === 'collections' && (
                <CollectionManager
                    apiKey={apiKey}
                    collections={collections}
                    onRefresh={fetchCollections}
                    onSelectCollection={setSelectedCollection}
                />
            )}
            {activeTab === 'visualizer' && (
                <PoincareVisualizer
                    apiKey={apiKey}
                    collections={collections}
                    selectedCollection={selectedCollection}
                />
            )}
            {activeTab === 'metrics' && (
                <SystemMetrics apiKey={apiKey} />
            )}
        </div>
    )
}
