import axios from "axios"

export const api = axios.create({
    baseURL: "/api",
})

export const setAuthToken = (token: string) => {
    api.defaults.headers.common["x-api-key"] = token
}

api.interceptors.response.use(
    (response) => response,
    (error) => {
        if (error.response?.status === 401) {
            // Only clear if we're not on login page already
            if (localStorage.getItem("hyperspace_api_key") && !window.location.pathname.includes("/login")) {
                localStorage.removeItem("hyperspace_api_key")
                window.location.href = "/login"
            }
        }
        return Promise.reject(error)
    }
)

const token = localStorage.getItem("hyperspace_api_key")
if (token) setAuthToken(token)

export const fetchStatus = async () => {
    try {
        const res = await api.get("/status")
        return res.data
    } catch {
        const res = await api.get("/cluster/status")
        return res.data
    }
}

