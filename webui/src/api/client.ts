const BASE_URL = '/api'

export class ApiError extends Error {
  status: number
  properties: Record<string, unknown>

  constructor(status: number, message: string, properties: Record<string, unknown> = {}) {
    super(message)
    this.status = status
    this.properties = properties
  }
}

function parseError(status: number, body: string): ApiError {
  try {
    const parsed = JSON.parse(body)
    const message = parsed.detail ?? parsed.message ?? parsed.error ?? body
    const { detail: _d, message: _m, error: _e, status: _s, title: _t, ...properties } = parsed
    return new ApiError(status, message, properties)
  } catch {
    return new ApiError(status, body || 'Request failed')
  }
}

async function request<T>(path: string, options?: RequestInit): Promise<T> {
  const response = await fetch(`${BASE_URL}${path}`, {
    ...options,
    headers: {
      'Accept': 'application/json',
      ...options?.headers,
    },
  })

  if (!response.ok) {
    throw parseError(response.status, await response.text())
  }

  if (response.status === 204) return undefined as T
  return response.json()
}

export function get<T>(path: string): Promise<T> {
  return request<T>(path)
}

export function post<T>(path: string): Promise<T> {
  return request<T>(path, { method: 'POST' })
}

export function del(path: string): Promise<void> {
  return request<void>(path, { method: 'DELETE' })
}
