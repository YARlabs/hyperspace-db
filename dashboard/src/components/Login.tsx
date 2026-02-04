import { useState } from 'react'

interface LoginProps {
    onLogin: (key: string) => void
    logo: string
}

export default function Login({ onLogin, logo }: LoginProps) {
    const [key, setKey] = useState('')
    const [error, setError] = useState('')

    const handleSubmit = async (e: React.FormEvent) => {
        e.preventDefault()

        // Test API key by calling list collections
        try {
            const res = await fetch('/api/collections', {
                headers: { 'x-api-key': key }
            })

            if (res.ok) {
                onLogin(key)
            } else {
                setError('Invalid API key')
            }
        } catch (err) {
            setError('Connection error')
        }
    }

    return (
        <div style={{
            display: 'flex',
            flexDirection: 'column',
            alignItems: 'center',
            justifyContent: 'center',
            minHeight: '100vh',
            gap: '20px'
        }}>
            <img src={logo} alt="HyperspaceDB" style={{ width: '100px' }} />
            <h1>HyperspaceDB Dashboard</h1>
            <form onSubmit={handleSubmit} style={{ display: 'flex', flexDirection: 'column', gap: '10px', width: '300px' }}>
                <input
                    type="password"
                    placeholder="API Key"
                    value={key}
                    onChange={e => setKey(e.target.value)}
                    style={{ padding: '10px', fontSize: '16px' }}
                />
                <button type="submit" style={{ padding: '10px', fontSize: '16px' }}>Login</button>
                {error && <p style={{ color: '#ff4444' }}>{error}</p>}
            </form>
            <p style={{ fontSize: '12px', color: '#666' }}>Default key: I_LOVE_HYPERSPACEDB</p>
        </div>
    )
}
