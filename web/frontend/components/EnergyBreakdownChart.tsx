'use client'

import React, { useMemo } from 'react'
import {
  BarChart,
  Bar,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  Legend,
  ResponsiveContainer,
} from 'recharts'
import { Run } from '@/lib/mock-data'

interface EnergyBreakdownChartProps {
  run: Run
}

export function EnergyBreakdownChart({ run }: EnergyBreakdownChartProps) {
  const chartData = useMemo(() => {
    // Create time buckets for better visualization
    const buckets: Record<string, { cpu: number; gpu: number; mem: number; igpu: number; count: number }> = {}

    run.records.forEach((record) => {
      const time = new Date(record.timestamp)
      const minute = Math.floor(time.getTime() / 60000) * 60000
      const key = new Date(minute).toLocaleTimeString('en-US', {
        hour: '2-digit',
        minute: '2-digit',
      })

      if (!buckets[key]) {
        buckets[key] = { cpu: 0, gpu: 0, mem: 0, igpu: 0, count: 0 }
      }

      buckets[key].cpu += record.cpu_energy
      buckets[key].gpu += record.gpu_energy
      buckets[key].mem += record.mem_energy
      buckets[key].igpu += record.igpu_energy
      buckets[key].count += 1
    })

    return Object.entries(buckets)
      .slice(-20) // Last 20 buckets
      .map(([time, data]) => ({
        time,
        cpu: Math.round(data.cpu / data.count),
        gpu: Math.round(data.gpu / data.count),
        mem: Math.round(data.mem / data.count),
        igpu: Math.round(data.igpu / data.count),
      }))
  }, [run.records])

  return (
    <div className="chart-container">
      <h3 className="text-lg font-semibold mb-4 text-foreground">
        Energy Breakdown by Component
      </h3>
      <ResponsiveContainer width="100%" height={300}>
        <BarChart data={chartData}>
          <CartesianGrid strokeDasharray="3 3" stroke="rgba(255,255,255,0.1)" />
          <XAxis
            dataKey="time"
            stroke="rgba(255,255,255,0.4)"
            style={{ fontSize: '12px' }}
          />
          <YAxis
            stroke="rgba(255,255,255,0.4)"
            style={{ fontSize: '12px' }}
            label={{ value: 'Watts', angle: -90, position: 'insideLeft' }}
          />
          <Tooltip
            contentStyle={{
              backgroundColor: 'rgba(0,0,0,0.8)',
              border: '1px solid rgba(255,255,255,0.2)',
              borderRadius: '8px',
            }}
          />
          <Legend wrapperStyle={{ fontSize: '12px' }} />
          <Bar
            dataKey="cpu"
            fill="hsl(160 84% 39%)"
            radius={[4, 4, 0, 0]}
            name="CPU Energy (W)"
            isAnimationActive={false}
          />
          <Bar
            dataKey="gpu"
            fill="hsl(49 89% 52%)"
            radius={[4, 4, 0, 0]}
            name="GPU Energy (W)"
            isAnimationActive={false}
          />
          <Bar
            dataKey="mem"
            fill="hsl(210 100% 50%)"
            radius={[4, 4, 0, 0]}
            name="Memory Energy (W)"
            isAnimationActive={false}
          />
          <Bar
            dataKey="igpu"
            fill="hsl(280 100% 50%)"
            radius={[4, 4, 0, 0]}
            name="iGPU Energy (W)"
            isAnimationActive={false}
          />
        </BarChart>
      </ResponsiveContainer>
    </div>
  )
}
