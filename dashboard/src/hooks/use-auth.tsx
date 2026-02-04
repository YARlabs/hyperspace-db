import React, { createContext, useContext, useState } from "react"
import { useLocation, Navigate } from "react-router-dom"

type AuthContextType = {
    apiKey: string | null
    setApiKey: (key: string | null) => void
    isAuthenticated: boolean
}

const AuthContext = createContext<AuthContextType>(null!)

export function AuthProvider({ children }: { children: React.ReactNode }) {
    const [apiKey, setApiKeyState] = useState<string | null>(
        localStorage.getItem("hyperspace_api_key")
    )

    const setApiKey = (key: string | null) => {
        if (key) {
            localStorage.setItem("hyperspace_api_key", key)
        } else {
            localStorage.removeItem("hyperspace_api_key")
        }
        setApiKeyState(key)
    }

    return (
        <AuthContext.Provider
            value={{ apiKey, setApiKey, isAuthenticated: !!apiKey }}
        >
            {children}
        </AuthContext.Provider>
    )
}

export function useAuth() {
    return useContext(AuthContext)
}

export function RequireAuth({ children }: { children: React.ReactElement }) {
    let auth = useAuth()
    let location = useLocation()

    if (!auth.isAuthenticated) {
        return <Navigate to="/login" state={{ from: location }} replace />
    }

    return children
}
