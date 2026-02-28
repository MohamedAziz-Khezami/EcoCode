'use client'

import { useEffect, useState } from 'react'
import Link from 'next/link'
import { ArrowLeft, Zap, Leaf } from 'lucide-react'
import { DashboardLayout } from '@/components/DashboardLayout'
import { EnergyChart } from '@/components/EnergyChart'
import { UtilizationChart } from '@/components/UtilizationChart'
import { EnergyBreakdownChart } from '@/components/EnergyBreakdownChart'
import { RunSystemInfo } from '@/components/RunSystemInfo'
import { Run } from '@/lib/mock-data'
import { fetchRunDetail } from '@/lib/api-client'
import { useSSE } from '@/hooks/use-sse'
import { format } from 'date-fns'

interface RunDetailPageProps {
  params: Promise<{ id: string }>
}

export default function RunDetailPage({ params }: RunDetailPageProps) {
  const [run, setRun] = useState<Run | null>(null)
  const [loading, setLoading] = useState(true)
  const [paramId, setParamId] = useState<string | null>(null)
  const { latestRecord } = useSSE()

  useEffect(() => {
    const initParams = async () => {
      const { id } = await params
      setParamId(id)
    }
    initParams()
  }, [params])

  // Update current run with real-time SSE data
  useEffect(() => {
    if (!latestRecord || !run || latestRecord.run_id !== paramId) return

    setRun((prevRun) => {
      if (!prevRun) return null

      // Check if the record already exists to avoid duplicates
      if (prevRun.records.some((r) => r.id === latestRecord.id)) return prevRun

      const updatedRecords = [...prevRun.records, latestRecord]
      const count = updatedRecords.length

      // Calculate new total energy incrementally (O(1))
      const newTotalEnergy =
        prevRun.totalEnergy +
        (latestRecord.cpu_energy +
          latestRecord.gpu_energy +
          latestRecord.mem_energy +
          latestRecord.igpu_energy)

      const energyKwh = newTotalEnergy / 1000

      // Incremental averages
      const avgCpuUsage = (prevRun.avgCpuUsage * (count - 1) + latestRecord.cpu_usage) / count
      const avgGpuUsage = (prevRun.avgGpuUsage * (count - 1) + latestRecord.gpu_usage) / count
      const avgMemUsage = (prevRun.avgMemUsage * (count - 1) + latestRecord.mem_usage) / count

      return {
        ...prevRun,
        records: updatedRecords,
        totalEnergy: newTotalEnergy,
        carbonFootprint: energyKwh * 0.475 * 1000,
        avgCpuUsage,
        avgGpuUsage,
        avgMemUsage,
        status: 'running', // Definitely running if we're getting live data
      }
    })
  }, [latestRecord, paramId])

  useEffect(() => {
    if (!paramId) return

    const fetchRun = async () => {
      try {
        const data = await fetchRunDetail(paramId)
        setRun(data)
      } catch (error) {
        console.error('Failed to fetch run:', error)
      } finally {
        setLoading(false)
      }
    }

    fetchRun()
  }, [paramId])

  if (loading) {
    return (
      <DashboardLayout>
        <div className="p-4 md:p-6 lg:p-8 w-full">
          <div className="h-96 glass rounded-lg animate-pulse" />
        </div>
      </DashboardLayout>
    )
  }

  if (!run) {
    return (
      <DashboardLayout>
        <div className="p-4 md:p-6 lg:p-8 w-full">
          <div className="text-center py-12">
            <p className="text-muted-foreground mb-4">Run not found</p>
            <Link
              href="/runs"
              className="inline-flex items-center gap-2 text-primary hover:text-primary/80 transition-colors"
            >
              <ArrowLeft className="w-4 h-4" />
              Back to runs
            </Link>
          </div>
        </div>
      </DashboardLayout>
    )
  }

  const statusColor =
    run.status === 'finished'
      ? 'bg-emerald-500/20 text-emerald-300'
      : run.status === 'running'
        ? 'bg-blue-500/20 text-blue-300'
        : 'bg-red-500/20 text-red-300'

  return (
    <DashboardLayout>
      <div className="p-4 md:p-6 lg:p-8 w-full">
        {/* Header with back button */}
        <div className="mb-6 md:mb-8">
          <Link
            href="/runs"
            className="inline-flex items-center gap-2 text-primary hover:text-primary/80 transition-colors mb-4"
          >
            <ArrowLeft className="w-4 h-4" />
            Back to runs
          </Link>

          <div className="flex flex-col md:flex-row md:items-start md:justify-between gap-3 md:gap-4">
            <div className="flex-1">
              <h1 className="text-2xl md:text-3xl lg:text-4xl font-bold text-foreground mb-2">
                {run.name}
              </h1>
              <p className="text-sm md:text-base text-muted-foreground">
                {format(new Date(run.timestamp), 'EEEE, MMMM d, yyyy HH:mm')}
              </p>
            </div>
            <span className={`inline-block px-3 md:px-4 py-2 rounded-lg font-medium whitespace-nowrap ${statusColor}`}>
              {run.status}
            </span>
          </div>
        </div>

        {/* System Info Grid */}
        <div className="mb-8">
          <RunSystemInfo run={run} />
        </div>

        {/* Charts Grid */}
        <div className="mb-8 space-y-4 md:space-y-6">
          <div className="grid grid-cols-1 lg:grid-cols-2 gap-4 md:gap-6">
            <EnergyChart data={run.records} />
            <UtilizationChart data={run.records} />
          </div>
        </div>

        {/* Energy Breakdown */}
        <div className="mb-8">
          <EnergyBreakdownChart run={run} />
        </div>

        {/* Summary Cards */}
        <div className="grid grid-cols-1 md:grid-cols-2 gap-4 md:gap-6">
          <div className="chart-container">
            <div className="flex items-start gap-4">
              <div className="p-3 rounded-lg bg-primary/10">
                <Zap className="w-6 h-6 text-primary" />
              </div>
              <div className="flex-1">
                <p className="text-sm text-muted-foreground mb-1">
                  Total Energy Consumption
                </p>
                <p className="text-3xl font-bold text-foreground">
                  {Math.round(run.totalEnergy).toLocaleString()}
                </p>
                <p className="text-xs text-muted-foreground mt-2">Wh</p>
              </div>
            </div>
          </div>

          {/* <div className="chart-container">
            <div className="flex items-start gap-4">
              <div className="p-3 rounded-lg bg-primary/10">
                <Leaf className="w-6 h-6 text-primary" />
              </div>
              <div className="flex-1">
                <p className="text-sm text-muted-foreground mb-1">
                  Carbon Footprint
                </p>
                <p className="text-3xl font-bold text-foreground">
                  {Math.round(run.carbonFootprint).toLocaleString()}
                </p>
                <p className="text-xs text-muted-foreground mt-2">
                  g CO₂e (World avg: 0.475 kg/kWh)
                </p>
              </div>
            </div>
          </div> */}
        </div>
      </div>
    </DashboardLayout>
  )
}
