import {useState, useEffect} from 'react';
import Dialog from '@jetbrains/ring-ui-built/components/dialog/dialog';
import Button from '@jetbrains/ring-ui-built/components/button/button';
import Header from '@jetbrains/ring-ui-built/components/island/header';
import Content from '@jetbrains/ring-ui-built/components/island/content';
import Panel from '@jetbrains/ring-ui-built/components/panel/panel';
import type {RepoConfig} from '../../types';
import {df} from './dialogStyles';

interface RepoDialogProps {
    show: boolean;
    onClose: () => void;
    onSubmit: (data: RepoConfig) => Promise<void>;
    initialData?: RepoConfig | null;
}

export const RepoDialog = ({show, onClose, onSubmit, initialData}: RepoDialogProps) => {
    const [name, setName] = useState('');
    const [url, setUrl] = useState('');
    const [loading, setLoading] = useState(false);
    const [error, setError] = useState<string | null>(null);

    const isEdit = !!initialData;

    useEffect(() => {
        if (show) {
            if (initialData) {
                setName(initialData.name);
                setUrl(initialData.url);
            } else {
                setName('');
                setUrl('');
            }
            setError(null);
            setLoading(false);
        }
    }, [show, initialData]);

    const handleSubmit = async (e: React.FormEvent) => {
        e.preventDefault();
        if (!name.trim()) {
            setError('Name is required');
            return;
        }
        if (!url.trim()) {
            setError('URL is required');
            return;
        }
        setLoading(true);
        setError(null);
        try {
            await onSubmit({
                name: name.trim(),
                url: url.trim(),
            });
            onClose();
        } catch (err) {
            setError(err instanceof Error ? err.message : 'Failed to save repository');
        } finally {
            setLoading(false);
        }
    };

    const handleClose = () => {
        if (!loading) {
            onClose();
        }
    };

    return (
        <Dialog
            show={show}
            label={isEdit ? 'Edit Repository' : 'Add Repository'}
            onCloseAttempt={handleClose}
            onOverlayClick={handleClose}
            onEscPress={handleClose}
            showCloseButton
            trapFocus
            dense
        >
            <form onSubmit={handleSubmit}>
                <Header>{isEdit ? 'Edit Repository' : 'Add Repository'}</Header>
                <Content className={df.form}>
                    <div className={df.group}>
                        <label htmlFor="repo-name" className={df.label}>
                            Name
                        </label>
                        <div className={df.control}>
                            <input
                                id="repo-name"
                                className={`${df.input} ${df.inputM} ${error && !name.trim() ? df.inputError : ''}`}
                                type="text"
                                value={name}
                                onChange={(e) => setName(e.target.value)}
                                placeholder="Enter repository name"
                                disabled={loading}
                                autoFocus
                            />
                        </div>
                    </div>

                    <div className={df.group}>
                        <label htmlFor="repo-url" className={df.label}>
                            URL
                        </label>
                        <div className={df.control}>
                            <input
                                id="repo-url"
                                className={`${df.input} ${error && !url.trim() ? df.inputError : ''}`}
                                type="text"
                                value={url}
                                onChange={(e) => setUrl(e.target.value)}
                                placeholder="git@github.com:org/repo.git"
                                disabled={loading}
                            />
                            {error && <div className={df.errorBubble}>{error}</div>}
                        </div>
                    </div>
                </Content>
                <Panel className="flex justify-end gap-2">
                    <Button primary type="submit" disabled={loading}>
                        {loading ? 'Saving...' : (isEdit ? 'Save' : 'Add')}
                    </Button>
                    <Button type="button" onClick={handleClose} disabled={loading}>
                        Cancel
                    </Button>
                </Panel>
            </form>
        </Dialog>
    );
};
