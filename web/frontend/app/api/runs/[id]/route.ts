import { getRunById } from '@/lib/mock-data'

export async function GET(
  request: Request,
  { params }: { params: Promise<{ id: string }> }
) {
  try {
    const { id } = await params
    const run = getRunById(id)

    if (!run) {
      return Response.json(
        { error: 'Run not found' },
        { status: 404 }
      )
    }

    return Response.json(run)
  } catch (error) {
    return Response.json(
      { error: 'Failed to fetch run' },
      { status: 500 }
    )
  }
}
