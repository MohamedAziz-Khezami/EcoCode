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
    time: format(new Date(record.timestamp), 'HH:mm:ss'),
    cpu: Number(record.cpu_energy.toFixed(2)),
    gpu: Number(record.gpu_energy.toFixed(2)),
    mem: Number(record.mem_energy.toFixed(2)),
    igpu: Number(record.igpu_energy.toFixed(2)),
    total: Number((record.cpu_energy + record.gpu_energy + record.mem_energy + record.igpu_energy).toFixed(2)),
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
          <Line
            type="monotone"
            dataKey="mem"
            stroke="hsl(210 100% 50%)"
            strokeWidth={2}
            dot={false}
            name="Memory Energy"
            isAnimationActive={true}
          />
          <Line
            type="monotone"
            dataKey="igpu"
            stroke="hsl(280 100% 50%)"
            strokeWidth={2}
            dot={false}
            name="iGPU Energy"
            isAnimationActive={true}
          />
        </LineChart>
      </ResponsiveContainer>
    </div>
  )
}
