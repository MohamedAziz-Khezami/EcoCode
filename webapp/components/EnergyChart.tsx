'use client'

import {
  LineChart,
  Line,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  ResponsiveContainer,
} from 'recharts'
import { Record } from '@/lib/mock-data'
import { format } from 'date-fns'

interface EnergyChartProps {
  data: Record[]
}

export function EnergyChart({ data }: EnergyChartProps) {
  const chartData = data.map((record) => ({
    time: format(new Date(record.timestamp), 'HH:mm'),
    cpu: Math.round(record.cpu_energy),
    gpu: Math.round(record.gpu_energy),
    total: Math.round(record.cpu_energy + record.gpu_energy),
  }))

  return (
    <div className="chart-container">
      <h3 className="text-lg font-semibold mb-4 text-foreground">
        Energy Consumption Over Time
      </h3>
      <ResponsiveContainer width="100%" height={300}>
        <LineChart data={chartData}>
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
            cursor={{ stroke: 'rgba(255,255,255,0.2)' }}
          />
          <Line
            type="monotone"
            dataKey="cpu"
            stroke="hsl(160 84% 39%)"
            strokeWidth={2}
            dot={false}
            name="CPU Energy"
            isAnimationActive={true}
          />
          <Line
            type="monotone"
            dataKey="gpu"
            stroke="hsl(49 89% 52%)"
            strokeWidth={2}
            dot={false}
            name="GPU Energy"
            isAnimationActive={true}
          />
        </LineChart>
      </ResponsiveContainer>
    </div>
  )
}
