'use client'

import { useEffect, useState } from 'react'
import Link from 'next/link'
import { ArrowLeft, Folder, Zap, Clock } from 'lucide-react'
import { DashboardLayout } from '@/components/DashboardLayout'
import { RecentRunsTable } from '@/components/RecentRunsTable'
import { ProjectDetail, fetchProjectRuns } from '@/lib/api-client'
import { MetricCard } from '@/components/MetricCard'

interface ProjectPageProps {
    params: Promise<{ id: string }>
}

export default function ProjectPage({ params }: ProjectPageProps) {
    const [project, setProject] = useState<ProjectDetail | null>(null)
    const [loading, setLoading] = useState(true)
    const [paramId, setParamId] = useState<string | null>(null)

    useEffect(() => {
        const initParams = async () => {
            const { id } = await params
            setParamId(id)
        }
        initParams()
    }, [params])

    useEffect(() => {
        if (!paramId) return

        const loadProject = async () => {
            try {
                const data = await fetchProjectRuns(paramId)
                setProject(data)
            } catch (error) {
                console.error('Failed to fetch project:', error)
            } finally {
                setLoading(false)
            }
        }
        loadProject()
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

    if (!project) {
        return (
            <DashboardLayout>
                <div className="p-4 md:p-6 lg:p-8 w-full">
                    <div className="text-center py-12">
                        <p className="text-muted-foreground mb-4">Project not found</p>
                        <Link
                            href="/"
                            className="inline-flex items-center gap-2 text-primary hover:text-primary/80 transition-colors"
                        >
                            <ArrowLeft className="w-4 h-4" />
                            Back to Dashboard
                        </Link>
                    </div>
                </div>
            </DashboardLayout>
        )
    }

    const totalCoreHours = project.runs.reduce((sum, r) => sum + r.totalCoreHours, 0)
    const avgCpuUsage = project.runs.length ? project.runs.reduce((sum, r) => sum + r.avgCpuUsage, 0) / project.runs.length : 0

    return (
        <DashboardLayout>
            <div className="p-4 md:p-6 lg:p-8 w-full">
                <div className="mb-6 md:mb-8">
                    <Link
                        href="/"
                        className="inline-flex items-center gap-2 text-primary hover:text-primary/80 transition-colors mb-4"
                    >
                        <ArrowLeft className="w-4 h-4" />
                        Back to Dashboard
                    </Link>

                    <div className="flex flex-col md:flex-row md:items-start md:justify-between gap-3 md:gap-4">
                        <div className="flex-1">
                            <h1 className="text-2xl md:text-3xl lg:text-4xl font-bold text-foreground mb-2 flex items-center gap-3">
                                <Folder className="w-8 h-8 text-primary" />
                                {project.name}
                            </h1>
                            <p className="text-sm md:text-base text-muted-foreground">
                                Project Dashboard
                            </p>
                        </div>
                    </div>
                </div>

                <div className="mb-8">
                    <h2 className="text-lg md:text-xl font-semibold text-foreground mb-4">
                        Project Overview
                    </h2>
                    <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-3 md:gap-4">
                        <MetricCard
                            icon={Zap}
                            label="Total Energy"
                            value={Math.round(project.totalEnergy).toLocaleString()}
                            unit="Wh"
                            loading={loading}
                        />
                        <MetricCard
                            icon={Clock}
                            label="Total Core Hours"
                            value={(totalCoreHours / 24).toFixed(2)}
                            unit="hrs"
                            loading={loading}
                        />
                        <MetricCard
                            icon={Folder}
                            label="Total Runs"
                            value={project.totalRuns.toString()}
                            unit=""
                            loading={loading}
                        />
                        <MetricCard
                            icon={Clock}
                            label="Avg CPU Usage"
                            value={avgCpuUsage.toFixed(1)}
                            unit="%"
                            loading={loading}
                        />
                    </div>
                </div>

                <RecentRunsTable runs={project.runs} />
            </div>
        </DashboardLayout>
    )
}
