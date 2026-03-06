'usesymotion-prefix) client';

import {useState} from 'react';
import type {CreateTaskRequest, TaskPriority} from '@/types/task';
import {api} from '@/lib/api';

export function CreateTaskForm() {
    const [formData, setFormData] = useState<CreateTaskRequest>({
        // repo_url: "git@github.com:k88936/test.git",
        // description:  "say hello world",
        repo_url: "git@github.com:k88936/parallel.git",
        description: "",
        base_branch: 'main',
        priority: 'normal',
    });
    const [loading, setLoading] = useState(false);
    const [error, setError] = useState<string | null>(null);

    const handleSubmit = async (e: React.FormEvent) => {
        e.preventDefault();
        setLoading(true);
        setError(null);

        try {
            await api.createTask(formData);
            setFormData({
                repo_url: '',
                description: '',
                base_branch: 'main',
                priority: 'normal',
            });
        } catch (err) {
            setError(err instanceof Error ? err.message : 'Failed to create task');
        } finally {
            setLoading(false);
        }
    };

    return (
        <form onSubmit={handleSubmit} className="bg-white shadow-md rounded px-8 pt-6 pb-8 mb-4">
            <h2 className="text-xl font-bold mb-4">Create New Task</h2>

            {error && (
                <div className="bg-red-100 border border-red-400 text-red-700 px-4 py-3 rounded mb-4">
                    {error}
                </div>
            )}

            <div className="mb-4">
                <label className="block text-gray-700 text-sm font-bold mb-2" htmlFor="repo_url">
                    Repository URL *
                </label>
                <input
                    id="repo_url"
                    type="text"
                    required
                    value={formData.repo_url}
                    onChange={(e) => setFormData({...formData, repo_url: e.target.value})}
                    className="shadow appearance-none border rounded w-full py-2 px-3 text-gray-700 leading-tight focus:outline-none focus:shadow-outline"
                />
            </div>

            <div className="mb-4">
                <label className="block text-gray-700 text-sm font-bold mb-2" htmlFor="description">
                    Description *
                </label>
                <textarea
                    id="description"
                    required
                    value={formData.description}
                    onChange={(e) => setFormData({...formData, description: e.target.value})}
                    className="shadow appearance-none border rounded w-full py-2 px-3 text-gray-700 leading-tight focus:outline-none focus:shadow-outline"
                    rows={4}
                />
            </div>

            <div className="mb-4">
                <label className="block text-gray-700 text-sm font-bold mb-2" htmlFor="base_branch">
                    Base Branch
                </label>
                <input
                    id="base_branch"
                    type="text"
                    value={formData.base_branch || ''}
                    onChange={(e) => setFormData({...formData, base_branch: e.target.value})}
                    className="shadow appearance-none border rounded w-full py-2 px-3 text-gray-700 leading-tight focus:outline-none focus:shadow-outline"
                    placeholder="main"
                />
            </div>

            <div className="mb-6">
                <label className="block text-gray-700 text-sm font-bold mb-2" htmlFor="priority">
                    Priority
                </label>
                <select
                    id="priority"
                    value={formData.priority || 'normal'}
                    onChange={(e) => setFormData({...formData, priority: e.target.value as TaskPriority})}
                    className="shadow appearance-none border rounded w-full py-2 px-3 text-gray-700 leading-tight focus:outline-none focus:shadow-outline"
                >
                    <option value="low">Low</option>
                    <option value="normal">Normal</option>
                    <option value="high">High</option>
                    <option value="urgent">Urgent</option>
                </select>
            </div>

            <div className="flex items-center justify-between">
                <button
                    type="submit"
                    disabled={loading}
                    className="bg-blue-500 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded focus:outline-none focus:shadow-outline disabled:opacity-50"
                >
                    {loading ? 'Creating...' : 'Create Task'}
                </button>
            </div>
        </form>
    );
}
