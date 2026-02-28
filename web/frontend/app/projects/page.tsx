'use client'

import { useEffect, useState } from 'react'
import { DashboardLayout } from '@/components/DashboardLayout'
import { ProjectsTable } from '@/components/ProjectsTable'
import { ProjectSummary, fetchAllProjects } from '@/lib/api-client'
import { Folder } from 'lucide-react'

export default function ProjectsPage() {
    const [projects, setProjects] = useState<ProjectSummary[]>([])
    const [loading, setLoading] = useState(true)

    useEffect(() => {
        const loadProjects = async () => {
            try {
                const data = await fetchAllProjects()
                setProjects(data)
            } catch (error) {
                console.error('Failed to load projects:', error)
            } finally {
                setLoading(false)
            }
        }
        loadProjects()
    }, [])

    return (
        <DashboardLayout>
            <div className="p-4 md:p-6 lg:p-8 w-full">
                <div className="mb-6 md:mb-8">
                    <h1 className="text-2xl md:text-3xl lg:text-4xl font-bold text-foreground mb-2 flex items-center gap-3">
                        <Folder className="w-8 h-8 text-primary" />
                        Projects
                    </h1>
                    <p className="text-sm md:text-base text-muted-foreground">
                        All registered EcoCode projects
                    </p>
                </div>

                <ProjectsTable projects={projects} />
            </div>
        </DashboardLayout>
    )
}
