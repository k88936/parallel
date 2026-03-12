import {useEffect, useState} from 'react';
import Dialog from '@jetbrains/ring-ui-built/components/dialog/dialog';
import Button from '@jetbrains/ring-ui-built/components/button/button';
import Select from '@jetbrains/ring-ui-built/components/select/select';
import Input from '@jetbrains/ring-ui-built/components/input/input';
import Header from '@jetbrains/ring-ui-built/components/island/header';
import Content from '@jetbrains/ring-ui-built/components/island/content';
import Panel from '@jetbrains/ring-ui-built/components/panel/panel';
import type {CreateTaskRequest, RepoConfig, SshKeyConfig, TaskPriority} from '../../types';
import {df} from './dialogStyles';
import Theme, {ThemeContext, ThemeProvider} from "@jetbrains/ring-ui-built/components/global/theme";

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

const PRIORITY_OPTIONS: Array<{ key: TaskPriority; label: string }> = [
    {key: 'low', label: 'Low'},
    {key: 'normal', label: 'Normal'},
    {key: 'high', label: 'High'},
    {key: 'urgent', label: 'Urgent'},
];

const generateDefaultTitle = () => {
    const now = new Date();
    const pad = (n: number) => n.toString().padStart(2, '0');
    return `Task ${now.getFullYear()}-${pad(now.getMonth() + 1)}-${pad(now.getDate())} ${pad(now.getHours())}:${pad(now.getMinutes())}:${pad(now.getSeconds())}`;
};

const isTaskPriority = (value: string): value is TaskPriority => PRIORITY_OPTIONS.some((option) => option.key === value);

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
    const [descriptionError, setDescriptionError] = useState<string | null>(null);
    const [repoRef, setRepoRef] = useState('');
    const [sshKeyRef, setSshKeyRef] = useState('');
    const [baseBranch, setBaseBranch] = useState('main');
    const [targetBranch, setTargetBranch] = useState('');
    const [priority, setPriority] = useState<TaskPriority>('normal');
    const [maxExecutionTime, setMaxExecutionTime] = useState('3600');
    const [labelValue, setLabelValue] = useState('');
    const [labels, setLabels] = useState<Record<string, "">>({});

    const resetForm = () => {
        setTitle(generateDefaultTitle());
        setDescription('');
        setDescriptionError(null);
        setRepoRef(repos[0]?.name || '');
        setSshKeyRef(sshKeys[0]?.name || '');
        setBaseBranch('main');
        setTargetBranch('');
        setPriority('normal');
        setMaxExecutionTime('3600');
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
        if (labelValue.trim()) {
            setLabels({...labels, [labelValue.trim()]: ""});
            setLabelValue('');
        }
    };

    const handleRemoveLabel = (key: string) => {
        const newLabels = {...labels};
        delete newLabels[key];
        setLabels(newLabels);
    };

    const handleSubmit = async (e: React.FormEvent) => {
        e.preventDefault();
        if (!description.trim()) {
            setDescriptionError('Description is required');
            return;
        }
        setDescriptionError(null);

        const data: CreateTaskRequest = {
            title: title.trim() || generateDefaultTitle(),
            repo_ref: repoRef,
            description: description.trim(),
            base_branch: baseBranch.trim() || null,
            target_branch: targetBranch.trim() || null,
            priority: priority === 'normal' ? null : priority,
            ssh_key_ref: sshKeyRef,
            max_execution_time: maxExecutionTime ? Number(maxExecutionTime) : null,
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

    const repoOptions = repos.map((r) => ({key: r.name, label: r.name}));
    const sshKeyOptions = sshKeys.map((k) => ({key: k.name, label: k.name}));
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
            closeButtonInside
            showCloseButton
            trapFocus
        >
            <form onSubmit={handleSubmit}>
                <Header>Create Task</Header>
                <Content className={df.form}>
                    <div className={df.group}>
                        <label htmlFor="task-title" className={df.label}>
                            Title
                        </label>
                        <div className={df.control}>
                            <input
                                id="task-title"
                                className={`${df.input} ${df.inputM}`}
                                type="text"
                                value={title}
                                onChange={(e) => setTitle(e.target.value)}
                                placeholder="Task title (default: current time)"
                                disabled={loading}
                            />
                        </div>
                    </div>

                    <div className={df.group}>
                        <label htmlFor="task-description" className={df.label}>
                            Description <span className="text-[var(--ring-error-color,#f00)]">*</span>
                        </label>
                        <div className={df.control}>
                            <textarea
                                id="task-description"
                                className={`${df.input} ${df.inputM} ${descriptionError ? df.inputError : ''}`}
                                value={description}
                                onChange={(e) => {
                                    setDescription(e.target.value);
                                    if (descriptionError && e.target.value.trim()) {
                                        setDescriptionError(null);
                                    }
                                }}
                                placeholder="Task description..."
                                disabled={loading}
                                rows={4}
                                style={{resize: 'vertical'}}
                            />
                            {descriptionError && <div className={df.errorBubble}>{descriptionError}</div>}
                        </div>
                    </div>

                    <div className={df.group}>
                        <label className={df.label}>Repository</label>
                        <div className={df.control}>
                            <Select
                                data={repoOptions}
                                selected={selectedRepo}
                                onSelect={(opt) => setRepoRef(opt?.key || '')}
                                disabled={loading || repos.length === 0}
                                type={Select.Type.INLINE}
                            />
                        </div>
                    </div>

                    <div className={df.group}>
                        <label className={df.label}>SSH Key</label>
                        <div className={df.control}>
                            <Select
                                data={sshKeyOptions}
                                selected={selectedSshKey}
                                onSelect={(opt) => setSshKeyRef(opt?.key || '')}
                                disabled={loading || sshKeys.length === 0}
                                type={Select.Type.INLINE}
                            />
                        </div>
                    </div>

                    <div className={df.group}>
                        <label htmlFor="task-base-branch" className={df.label}>
                            Base Branch
                        </label>
                        <div className={df.control}>
                            <input
                                id="task-base-branch"
                                className={`${df.input} ${df.inputM}`}
                                type="text"
                                value={baseBranch}
                                onChange={(e) => setBaseBranch(e.target.value)}
                                placeholder="main"
                                disabled={loading}
                            />
                        </div>
                    </div>

                    <div className={df.group}>
                        <label htmlFor="task-target-branch" className={df.label}>
                            Target Branch
                        </label>
                        <div className={df.control}>
                            <input
                                id="task-target-branch"
                                className={`${df.input} ${df.inputM}`}
                                type="text"
                                value={targetBranch}
                                onChange={(e) => setTargetBranch(e.target.value)}
                                placeholder="Auto-generated if empty"
                                disabled={loading}
                            />
                        </div>
                    </div>

                    <div className={df.group}>
                        <label className={df.label}>Priority</label>
                        <div className={df.control}>
                            <Select
                                data={PRIORITY_OPTIONS}
                                selected={selectedPriority}
                                onSelect={(opt) => setPriority(opt && isTaskPriority(opt.key) ? opt.key : 'normal')}
                                disabled={loading}
                                type={Select.Type.INLINE}
                            />
                        </div>
                    </div>

                    <div className={df.group}>
                        <label htmlFor="task-max-time" className={df.label}>
                            Max Execution Time (s)
                        </label>
                        <div className={df.control}>
                            <input
                                id="task-max-time"
                                className={`${df.input} ${df.inputM}`}
                                type="number"
                                value={maxExecutionTime}
                                onChange={(e) => setMaxExecutionTime(e.target.value)}
                                placeholder="3600"
                                disabled={loading}
                                min={1}
                            />
                        </div>
                    </div>

                    <div className={df.group}>
                        <label className={df.label}>Required Labels</label>
                        <div className={df.control}>
                            <div className="flex gap-2 mb-2">
                                <Input
                                    value={labelValue}
                                    onChange={(e) => setLabelValue(e.target.value)}
                                    placeholder="Value"
                                    disabled={loading}
                                />
                                <Button type="button" onClick={handleAddLabel} disabled={loading || !labelValue.trim()}>
                                    Add
                                </Button>
                            </div>
                            {Object.entries(labels).map(([key, value]) => (
                                <div key={key} className="flex items-center gap-2 mb-1">
                                    <span className="px-2 py-0.5 bg-[#333] rounded text-sm">
                                        {key}: {value}
                                    </span>
                                    <Button danger inline type="button" onClick={() => handleRemoveLabel(key)}
                                            disabled={loading}>
                                        Remove
                                    </Button>
                                </div>
                            ))}
                        </div>
                    </div>

                    {error && (
                        <div className={df.group}>
                            <div className={df.label}/>
                            <div className={df.control}>
                                <div className={df.errorBubble}>{error}</div>
                            </div>
                        </div>
                    )}
                </Content>
                <Panel className="flex justify-end gap-2">
                    <Button primary type="submit" disabled={loading}>
                        {loading ? 'Creating...' : 'Create Task'}
                    </Button>
                    <Button type="button" onClick={handleClose} disabled={loading}>
                        Cancel
                    </Button>
                </Panel>
            </form>
        </Dialog>
    );
};
