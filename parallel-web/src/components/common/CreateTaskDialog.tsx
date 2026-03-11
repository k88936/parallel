import { useState, useEffect } from 'react';
import Dialog from '@jetbrains/ring-ui-built/components/dialog/dialog';
import Button from '@jetbrains/ring-ui-built/components/button/button';
import Select from '@jetbrains/ring-ui-built/components/select/select';
import Input from '@jetbrains/ring-ui-built/components/input/input';
import type { RepoConfig, SshKeyConfig, TaskPriority, CreateTaskRequest } from '../../types';
import './DialogForm.css';

interface CreateTaskDialogProps {
    show: boolean;
    projectId: string;
    repos: RepoConfig[];
    sshKeys: SshKeyConfig[];
    onClose: () => void;
    onSubmit: (data: CreateTaskRequest) => Promise<void>;
    loading?: boolean;
    error?: string | null;
}

const PRIORITY_OPTIONS = [
    { key: 'low', label: 'Low' },
    { key: 'normal', label: 'Normal' },
    { key: 'high', label: 'High' },
    { key: 'urgent', label: 'Urgent' },
];

const generateDefaultTitle = () => {
    const now = new Date();
    const pad = (n: number) => n.toString().padStart(2, '0');
    return `Task ${now.getFullYear()}-${pad(now.getMonth() + 1)}-${pad(now.getDate())} ${pad(now.getHours())}:${pad(now.getMinutes())}:${pad(now.getSeconds())}`;
};

export const CreateTaskDialog = ({
    show,
    projectId,
    repos,
    sshKeys,
    onClose,
    onSubmit,
    loading = false,
    error = null,
}: CreateTaskDialogProps) => {
    const [title, setTitle] = useState('');
    const [description, setDescription] = useState('');
    const [repoRef, setRepoRef] = useState('');
    const [sshKeyRef, setSshKeyRef] = useState('');
    const [baseBranch, setBaseBranch] = useState('main');
    const [targetBranch, setTargetBranch] = useState('');
    const [priority, setPriority] = useState<TaskPriority>('normal');
    const [maxExecutionTime, setMaxExecutionTime] = useState('3600');
    const [labelKey, setLabelKey] = useState('');
    const [labelValue, setLabelValue] = useState('');
    const [labels, setLabels] = useState<Record<string, string>>({});

    const resetForm = () => {
        setTitle(generateDefaultTitle());
        setDescription('');
        setRepoRef(repos[0]?.name || '');
        setSshKeyRef(sshKeys[0]?.name || '');
        setBaseBranch('main');
        setTargetBranch('');
        setPriority('normal');
        setMaxExecutionTime('3600');
        setLabelKey('');
        setLabelValue('');
        setLabels({});
    };

    useEffect(() => {
        if (show) {
            resetForm();
        }
        // eslint-disable-next-line react-hooks/exhaustive-deps
    }, [show]);

    const handleAddLabel = () => {
        if (labelKey.trim() && labelValue.trim()) {
            setLabels({ ...labels, [labelKey.trim()]: labelValue.trim() });
            setLabelKey('');
            setLabelValue('');
        }
    };

    const handleRemoveLabel = (key: string) => {
        const newLabels = { ...labels };
        delete newLabels[key];
        setLabels(newLabels);
    };

    const handleSubmit = async (e: React.FormEvent) => {
        e.preventDefault();
        if (!description.trim()) {
            return;
        }

        const data: CreateTaskRequest = {
            title: title.trim() || generateDefaultTitle(),
            repo_ref: repoRef,
            description: description.trim(),
            base_branch: baseBranch.trim() || null,
            target_branch: targetBranch.trim() || null,
            priority: priority === 'normal' ? null : priority,
            ssh_key_ref: sshKeyRef,
            max_execution_time: maxExecutionTime ? Number(maxExecutionTime): null,
            project_id: projectId,
            required_labels: Object.keys(labels).length > 0 ? labels : {},
        };

        await onSubmit(data);
    };

    const handleClose = () => {
        if (!loading) {
            onClose();
        }
    };

    const repoOptions = repos.map((r) => ({ key: r.name, label: r.name }));
    const sshKeyOptions = sshKeys.map((k) => ({ key: k.name, label: k.name }));
    const selectedRepo = repoOptions.find((o) => o.key === repoRef);
    const selectedSshKey = sshKeyOptions.find((o) => o.key === sshKeyRef);
    const selectedPriority = PRIORITY_OPTIONS.find((o) => o.key === priority);

    return (
        <Dialog
            show={show}
            label="Create Task"
            onCloseAttempt={handleClose}
            onOverlayClick={handleClose}
            onEscPress={handleClose}
            showCloseButton
            trapFocus
            dense
        >
            <div style={{ width: 520 }}>
                <form className="ring-form" onSubmit={handleSubmit}>
                    <span className="ring-form__title">Create Task</span>

                    <div className="ring-form__group">
                        <label htmlFor="task-title" className="ring-form__label">
                            Title
                        </label>
                        <div className="ring-form__control">
                            <input
                                id="task-title"
                                className="ring-input ring-input-size_m"
                                type="text"
                                value={title}
                                onChange={(e) => setTitle(e.target.value)}
                                placeholder="Task title (default: current time)"
                                disabled={loading}
                            />
                        </div>
                    </div>

                    <div className="ring-form__group">
                        <label htmlFor="task-description" className="ring-form__label">
                            Description <span style={{ color: 'red' }}>*</span>
                        </label>
                        <div className="ring-form__control">
                            <textarea
                                id="task-description"
                                className="ring-input ring-input-size_m"
                                value={description}
                                onChange={(e) => setDescription(e.target.value)}
                                placeholder="Task description..."
                                disabled={loading}
                                rows={4}
                                style={{ resize: 'vertical' }}
                            />
                        </div>
                    </div>

                    <div className="ring-form__group">
                        <label className="ring-form__label">Repository</label>
                        <div className="ring-form__control">
                            <Select
                                data={repoOptions}
                                selected={selectedRepo}
                                onSelect={(opt) => setRepoRef(opt?.key || '')}
                                disabled={loading || repos.length === 0}
                                type={Select.Type.INLINE}
                            />
                        </div>
                    </div>

                    <div className="ring-form__group">
                        <label className="ring-form__label">SSH Key</label>
                        <div className="ring-form__control">
                            <Select
                                data={sshKeyOptions}
                                selected={selectedSshKey}
                                onSelect={(opt) => setSshKeyRef(opt?.key || '')}
                                disabled={loading || sshKeys.length === 0}
                                type={Select.Type.INLINE}
                            />
                        </div>
                    </div>

                    <div className="ring-form__group">
                        <label htmlFor="task-base-branch" className="ring-form__label">
                            Base Branch
                        </label>
                        <div className="ring-form__control">
                            <input
                                id="task-base-branch"
                                className="ring-input ring-input-size_m"
                                type="text"
                                value={baseBranch}
                                onChange={(e) => setBaseBranch(e.target.value)}
                                placeholder="main"
                                disabled={loading}
                            />
                        </div>
                    </div>

                    <div className="ring-form__group">
                        <label htmlFor="task-target-branch" className="ring-form__label">
                            Target Branch
                        </label>
                        <div className="ring-form__control">
                            <input
                                id="task-target-branch"
                                className="ring-input ring-input-size_m"
                                type="text"
                                value={targetBranch}
                                onChange={(e) => setTargetBranch(e.target.value)}
                                placeholder="Auto-generated if empty"
                                disabled={loading}
                            />
                        </div>
                    </div>

                    <div className="ring-form__group">
                        <label className="ring-form__label">Priority</label>
                        <div className="ring-form__control">
                            <Select
                                data={PRIORITY_OPTIONS}
                                selected={selectedPriority}
                                onSelect={(opt) => setPriority(opt?.key as TaskPriority || 'normal')}
                                disabled={loading}
                                type={Select.Type.INLINE}
                            />
                        </div>
                    </div>

                    <div className="ring-form__group">
                        <label htmlFor="task-max-time" className="ring-form__label">
                            Max Execution Time (s)
                        </label>
                        <div className="ring-form__control">
                            <input
                                id="task-max-time"
                                className="ring-input ring-input-size_m"
                                type="number"
                                value={maxExecutionTime}
                                onChange={(e) => setMaxExecutionTime(e.target.value)}
                                placeholder="3600"
                                disabled={loading}
                                min={1}
                            />
                        </div>
                    </div>

                    <div className="ring-form__group">
                        <label className="ring-form__label">Required Labels</label>
                        <div className="ring-form__control">
                            <div style={{ display: 'flex', gap: 8, marginBottom: 8 }}>
                                <Input
                                    value={labelKey}
                                    onChange={(e) => setLabelKey(e.target.value)}
                                    placeholder="Key"
                                    disabled={loading}
                                />
                                <Input
                                    value={labelValue}
                                    onChange={(e) => setLabelValue(e.target.value)}
                                    placeholder="Value"
                                    disabled={loading}
                                />
                                <Button onClick={handleAddLabel} disabled={loading || !labelKey.trim() || !labelValue.trim()}>
                                    Add
                                </Button>
                            </div>
                            {Object.entries(labels).map(([key, value]) => (
                                <div key={key} style={{ display: 'flex', alignItems: 'center', gap: 8, marginBottom: 4 }}>
                                    <span className="ring-label" style={{ padding: '2px 8px', background: '#eee', borderRadius: 4 }}>
                                        {key}: {value}
                                    </span>
                                    <Button danger inline onClick={() => handleRemoveLabel(key)} disabled={loading}>
                                        Remove
                                    </Button>
                                </div>
                            ))}
                        </div>
                    </div>

                    {error && (
                        <div className="ring-form__group">
                            <div className="ring-error-bubble active">{error}</div>
                        </div>
                    )}

                    <div className="ring-form__footer">
                        <Button primary type="submit" disabled={loading || !description.trim()}>
                            {loading ? 'Creating...' : 'Create Task'}
                        </Button>
                        <Button onClick={handleClose} disabled={loading}>
                            Cancel
                        </Button>
                    </div>
                </form>
            </div>
        </Dialog>
    );
};
