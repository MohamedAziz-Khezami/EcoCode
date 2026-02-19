'use client'

import {
  AreaChart,
  Area,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  ResponsiveContainer,
} from 'recharts'
import { Record } from '@/lib/mock-data'
import { format } from 'date-fns'

interface UtilizationChartProps {
  data: Record[]
}

export function UtilizationChart({ data }: UtilizationChartProps) {
  const chartData = data.map((record) => ({
    time: format(new Date(record.timestamp), 'HH:mm'),
    cpu: Math.round(record.cpu_usage),
    gpu: Math.round(record.gpu_usage),
  }))

  return (
    <div className="chart-container">
      <h3 className="text-lg font-semibold mb-4 text-foreground">
        CPU & GPU Utilization
      </h3>
      <ResponsiveContainer width="100%" height={300}>
        <AreaChart data={chartData}>
          <defs>
            <linearGradient id="colorCpu" x1="0" y1="0" x2="0" y2="1">
              <stop offset="5%" stopColor="hsl(160 84% 39%)" stopOpacity={0.3} />
              <stop offset="95%" stopColor="hsl(160 84% 39%)" stopOpacity={0} />
            </linearGradient>
            <linearGradient id="colorGpu" x1="0" y1="0" x2="0" y2="1">
              <stop offset="5%" stopColor="hsl(49 89% 52%)" stopOpacity={0.3} />
              <stop offset="95%" stopColor="hsl(49 89% 52%)" stopOpacity={0} />
            </linearGradient>
          </defs>
          <CartesianGrid strokeDasharray="3 3" stroke="rgba(255,255,255,0.1)" />
          <XAxis
            dataKey="time"
            stroke="rgba(255,255,255,0.4)"
            style={{ fontSize: '12px' }}
          />
          <YAxis
            stroke="rgba(255,255,255,0.4)"
            style={{ fontSize: '12px' }}
            label={{ value: 'Usage %', angle: -90, position: 'insideLeft' }}
            domain={[0, 100]}
          />
          <Tooltip
            contentStyle={{
              backgroundColor: 'rgba(0,0,0,0.8)',
              border: '1px solid rgba(255,255,255,0.2)',
              borderRadius: '8px',
            }}
            cursor={{ stroke: 'rgba(255,255,255,0.2)' }}
          />
          <Area
            type="monotone"
            dataKey="cpu"
            stroke="hsl(160 84% 39%)"
            fillOpacity={1}
            fill="url(#colorCpu)"
            name="CPU Usage"
            isAnimationActive={true}
          />
          <Area
            type="monotone"
            dataKey="gpu"
            stroke="hsl(49 89% 52%)"
            fillOpacity={1}
            fill="url(#colorGpu)"
            name="GPU Usage"
            isAnimationActive={true}
          />
        </AreaChart>
      </ResponsiveContainer>
    </div>
  )
}
