/** Estado visual recibido de Rust; revision evita sobrescribir eventos nuevos con snapshots viejos. */
const jobs = new Map();
export const isConverting = job => ['queued', 'preparing', 'converting'].includes(job?.status);
export function rememberConversion(job) {
    if (!job?.path) return false;
    if ((jobs.get(job.path)?.revision ?? -1) > (job.revision ?? 0)) return false;
    jobs.set(job.path, job);
    if (jobs.size > 1024) {
        for (const [path, value] of jobs) {
            if (!isConverting(value)) jobs.delete(path);
            if (jobs.size <= 1024) break;
        }
    }
    return true;
}
export function conversionJobs() { return [...jobs.values()]; }
export function conversionFor(path) { return jobs.get(path); }
