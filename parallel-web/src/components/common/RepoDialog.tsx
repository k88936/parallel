import {useState, useEffect} from 'react';
import Dialog from '@jetbrains/ring-ui-built/components/dialog/dialog';
import Button from '@jetbrains/ring-ui-built/components/button/button';
import type {RepoConfig} from '../../types';
import './DialogForm.css';

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
            <div>
                <form className="ring-form" onSubmit={handleSubmit}>
                    <span className="ring-form__title">{isEdit ? 'Edit Repository' : 'Add Repository'}</span>

                    <div className="ring-form__group">
                        <label htmlFor="repo-name" className="ring-form__label">
                            Name
                        </label>
                        <div className="ring-form__control">
                            <input
                                id="repo-name"
                                className={`ring-input ring-input-size_m ${error && !name.trim() ? 'ring-input_error' : ''}`}
                                type="text"
                                value={name}
                                onChange={(e) => setName(e.target.value)}
                                placeholder="Enter repository name"
                                disabled={loading}
                                autoFocus
                            />
                        </div>
                    </div>

                    <div className="ring-form__group">
                        <label htmlFor="repo-url" className="ring-form__label">
                            URL
                        </label>
                        <div className="ring-form__control">
                            <input
                                id="repo-url"
                                className={`ring-input ${error && !url.trim() ? 'ring-input_error' : ''}`}
                                type="text"
                                value={url}
                                onChange={(e) => setUrl(e.target.value)}
                                placeholder="git@github.com:org/repo.git"
                                disabled={loading}
                            />
                            {error && <div className="ring-error-bubble active">{error}</div>}
                        </div>
                    </div>

                    <div className="ring-form__footer">
                        <Button primary type="submit" disabled={loading}>
                            {loading ? 'Saving...' : (isEdit ? 'Save' : 'Add')}
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
