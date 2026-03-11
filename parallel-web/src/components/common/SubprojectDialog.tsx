import {useState, useEffect} from 'react';
import Dialog from '@jetbrains/ring-ui-built/components/dialog/dialog';
import Button from '@jetbrains/ring-ui-built/components/button/button';
import type {CreateProjectRequest} from '../../types';
import './DialogForm.css';

interface SubprojectDialogProps {
    show: boolean;
    parentId: string;
    onClose: () => void;
    onSubmit: (data: CreateProjectRequest) => Promise<void>;
}

export const SubprojectDialog = ({show, parentId, onClose, onSubmit}: SubprojectDialogProps) => {
    const [name, setName] = useState('');
    const [loading, setLoading] = useState(false);
    const [error, setError] = useState<string | null>(null);

    useEffect(() => {
        if (show) {
            setName('');
            setError(null);
            setLoading(false);
        }
    }, [show]);

    const handleSubmit = async (e: React.FormEvent) => {
        e.preventDefault();
        if (!name.trim()) {
            setError('Name is required');
            return;
        }
        setLoading(true);
        setError(null);
        try {
            await onSubmit({
                name: name.trim(),
                repos: [],
                ssh_keys: [],
                parent_id: parentId,
            });
            onClose();
        } catch (err) {
            setError(err instanceof Error ? err.message : 'Failed to create subproject');
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
            label="Add Subproject"
            onCloseAttempt={handleClose}
            onOverlayClick={handleClose}
            onEscPress={handleClose}
            showCloseButton
            trapFocus
            dense
        >
            <div>
                <form className="ring-form" onSubmit={handleSubmit}>
                    <span className="ring-form__title">Add Subproject</span>

                    <div className="ring-form__group">
                        <label htmlFor="subproject-name" className="ring-form__label">
                            Name
                        </label>
                        <div className="ring-form__control">
                            <input
                                id="subproject-name"
                                className={`ring-input ring-input-size_m ${error ? 'ring-input_error' : ''}`}
                                type="text"
                                value={name}
                                onChange={(e) => setName(e.target.value)}
                                placeholder="Enter subproject name"
                                disabled={loading}
                                autoFocus
                            />
                            {error && <div className="ring-error-bubble active">{error}</div>}
                        </div>
                    </div>

                    <div className="ring-form__footer">
                        <Button primary type="submit" disabled={loading}>
                            {loading ? 'Creating...' : 'Create'}
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
