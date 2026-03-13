import {Fragment} from 'react';
import Dialog from '@jetbrains/ring-ui-built/components/dialog/dialog';
import Panel from '@jetbrains/ring-ui-built/components/panel/panel';
import Button from '@jetbrains/ring-ui-built/components/button/button';
import IslandHeader from '@jetbrains/ring-ui-built/components/island/header';
import IslandContent from '@jetbrains/ring-ui-built/components/island/content';
import Heading from '@jetbrains/ring-ui-built/components/heading/heading';
import Loader from '@jetbrains/ring-ui-built/components/loader/loader';
import Text from '@jetbrains/ring-ui-built/components/text/text';
import Tag from '@jetbrains/ring-ui-built/components/tag/tag';
import Link from '@jetbrains/ring-ui-built/components/link/link';
import Group from '@jetbrains/ring-ui-built/components/group/group';
import clipboard from '@jetbrains/ring-ui-built/components/clipboard/clipboard';
import Markdown from '@jetbrains/ring-ui-built/components/markdown/markdown';
import MarkdownIt from 'markdown-it';
import {highlight} from '@jetbrains/ring-ui-built/components/code/code';
import type {Task, ReviewData, FeedbackType} from '../../types';
import ScrollableSection from "@jetbrains/ring-ui-built/components/scrollable-section/scrollable-section";

const markdownIt = new MarkdownIt('commonmark', {
    html: false,
    highlight(str, lang) {
        if (lang && highlight.getLanguage(lang)) {
            return highlight.highlight(str, {
                language: lang
            }).value;
        }
        return '';
    }
}).enable('table');

export interface ReviewDialogProps {
    show: boolean;
    task: Task | null;
    reviewData: ReviewData | null;
    loading: boolean;
    feedbackMessage: string;
    feedbackError: string | null;
    submittingType: FeedbackType | null;
    onClose: () => void;
    onFeedbackChange: (message: string) => void;
    onSubmitFeedback: (feedbackType: FeedbackType) => void;
}

export const ReviewDialog = ({
    show,
    task,
    reviewData,
    loading,
    feedbackMessage,
    feedbackError,
    submittingType,
    onClose,
    onFeedbackChange,
    onSubmitFeedback,
}: ReviewDialogProps) => {
    const checkoutCommand = task
        ? `git fetch && git checkout ${task.target_branch}`
        : '';

    const renderCheckoutCommand = () => (
        <Fragment>
            <Group>
                <code
                    className="block rounded bg-[var(--ring-sidebar-background-color,#1e1e1e)] px-3 py-2 font-mono text-xs break-all">
                    {checkoutCommand}
                </code>
            </Group>
            <Group>
                <Link
                    onClick={() => clipboard.copyText(checkoutCommand, 'Command copied!', 'Command copying error')}
                    pseudo
                >
                    Copy
                </Link>
            </Group>
        </Fragment>
    );

    return (
        <Dialog
            show={show}
            label="Review Task"
            onCloseAttempt={onClose}
            onOverlayClick={onClose}
            onEscPress={onClose}
            closeButtonInside
            showCloseButton
            trapFocus
        >
            <IslandHeader>
                {task ? `Review ${task.title}` : 'Review Task'}
            </IslandHeader>
            <IslandContent className="p-4">
                {task ? (
                    <Group className="flex flex-col gap-4">
                        <Group>
                            <Heading level={4}>Checkout in local IDE</Heading>
                            <Group className="mt-2 flex flex-col gap-2">
                                {renderCheckoutCommand()}
                            </Group>
                        </Group>

                        <Group>
                            <Heading level={4}>Messages</Heading>
                            {loading && !reviewData ? (
                                <Group className="mt-2">
                                    <Loader/>
                                </Group>
                            ) : reviewData && reviewData.messages.length > 0 ? (
                                <ScrollableSection className="mt-2 max-h-[320px] overflow-y-auto">
                                    {reviewData.messages.map((message, index) => (
                                        <Group
                                            key={`${message.timestamp}-${index}`}
                                            className="mb-2 rounded  p-3 last:mb-0"
                                        >
                                            <Group className="mb-2 flex items-center gap-2">
                                                <Tag>{message.role}</Tag>
                                                <span className="text-xs text-gray-400">
                                                     {new Date(message.timestamp).toLocaleString()}
                                                 </span>
                                            </Group>
                                            <Markdown>
                                                <div dangerouslySetInnerHTML={{
                                                    __html: markdownIt.render(message.content),
                                                }}/>
                                            </Markdown>
                                        </Group>
                                    ))}
                                </ScrollableSection>
                            ) : (
                                <Text className="mt-2">No review messages available yet.</Text>
                            )}
                        </Group>

                        <Group>
                            <Heading level={4}>Feedback</Heading>
                            <Group className="mt-2">
                                 <textarea
                                     className="min-h-[120px] w-full rounded border  px-[10px] py-[6px] text-[13px] focus:outline-none disabled:opacity-50 disabled:cursor-not-allowed"
                                     value={feedbackMessage}
                                     onChange={(event) => onFeedbackChange(event.target.value)}
                                     placeholder="Add optional review feedback for the worker"
                                     disabled={submittingType !== null}
                                 />
                            </Group>
                            {feedbackError && (
                                <Text className="mt-2 text-red-500">
                                    {feedbackError}
                                </Text>
                            )}
                        </Group>
                    </Group>
                ) : (
                    <Text>No review task selected.</Text>
                )}
            </IslandContent>
            <Panel className="flex justify-end gap-2">
                <Button
                    primary
                    onClick={() => onSubmitFeedback('approve')}
                    disabled={!task || submittingType !== null}
                >
                    {submittingType === 'approve' ? 'Approving...' : 'Approve'}
                </Button>
                <Button
                    onClick={() => onSubmitFeedback('request_changes')}
                    disabled={!task || submittingType !== null}
                >
                    {submittingType === 'request_changes' ? 'Sending...' : 'Request Changes'}
                </Button>
                <Button
                    danger
                    onClick={() => onSubmitFeedback('abort')}
                    disabled={!task || submittingType !== null}
                >
                    {submittingType === 'abort' ? 'Aborting...' : 'Abort'}
                </Button>
                <Button onClick={onClose} disabled={submittingType !== null}>
                    Close
                </Button>
            </Panel>
        </Dialog>
    );
};
