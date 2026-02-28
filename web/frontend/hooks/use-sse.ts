'use client'

import { useEffect, useState } from 'react'
import { MetricRecord } from '@/lib/mock-data'

const API_BASE_URL = process.env.NEXT_PUBLIC_API_URL || 'http://localhost:3001'

export function useSSE() {
    const [latestRecord, setLatestRecord] = useState<MetricRecord | null>(null)
    const [error, setError] = useState<string | null>(null)

    useEffect(() => {
        const eventSource = new EventSource(`${API_BASE_URL}/api/sse`)

        eventSource.onmessage = (event) => {
            try {
                const data = event.data
                if (data.startsWith('New Project: ')) {
                    const jsonStr = data.replace('New Project: ', '')
                    const record = JSON.parse(jsonStr)

                    // Map backend record to frontend MetricRecord type
                    const mappedRecord: MetricRecord = {
                        id: record.id.toString(),
                        run_id: record.run_id.toString(),
                        pid: record.pid,
                        timestamp: new Date(record.timestamp).getTime(),
                        cpu_usage: record.cpu_usage,
                        cpu_energy: record.cpu_energy,
                        gpu_usage: record.gpu_usage,
                        gpu_energy: record.gpu_energy,
                        mem_usage: record.mem_usage,
                        mem_energy: record.mem_energy,
                        igpu_usage: record.igpu_usage,
                        igpu_energy: record.igpu_energy,
                    }

                    setLatestRecord(mappedRecord)
                }
            } catch (err) {
                console.error('Failed to parse SSE message:', err)
            }
        }

        eventSource.onerror = (err) => {
            console.error('SSE connection error:', err)
            setError('Connection lost. Reconnecting...')
            eventSource.close()

            // Reconnect after 5 seconds
            setTimeout(() => {
                setError(null)
            }, 5000)
        }

        return () => {
            eventSource.close()
        }
    }, [])

    return { latestRecord, error }
}
