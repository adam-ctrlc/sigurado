import type { Guide } from '$lib/guides/types';

export const facultyGuide: Guide = {
	slug: 'faculty',
	label: 'For faculty',
	title: 'Overseeing the stockroom as faculty',
	audience: 'Instructors and lab staff who supervise the room but do not administer it.',
	intro:
		'You can see everything the readers recorded and everything the students wrote down, and you can act on the gaps between the two. You cannot create accounts, pair readers, or change who receives text alerts: those belong to the administrator. Everything below is what you can do, in the order you will need it.',
	sections: [
		{
			title: 'Know what you are looking at',
			summary:
				'Sigurado records two different kinds of fact, and telling them apart is the whole skill.',
			steps: [
				{
					action: 'Treat reader events as fact.',
					detail:
						'A door scan, a cabinet unlock, a refusal. These come from the hardware and nobody can edit them.'
				},
				{
					action: 'Treat checkouts as claims.',
					detail:
						'A person typed these in. They are only trustworthy because recording one requires the code the cabinet showed at the moment it opened.'
				},
				{
					action: 'Read a flag as the gap between the two.',
					detail:
						'An opening with no checkout behind it is the case the system exists to surface, and the flags page names it for you.'
				}
			]
		},
		{
			title: 'Sign in and find your way around',
			summary: 'Three pages carry almost everything you need.',
			steps: [
				{
					action: 'Sign in with your username or email and your password.',
					click: 'Sign in'
				},
				{
					action: 'Open the dashboard for the state of the room right now.',
					click: 'Dashboard, in the sidebar under Monitoring'
				},
				{
					action: 'Open the audit log for the complete history.',
					click: 'Audit Log, in the sidebar under Monitoring'
				},
				{
					action: 'Open the flags page for the things that do not add up.',
					click: 'Flags, in the sidebar under Monitoring'
				},
				{
					action: 'Change your own password and details whenever you like.',
					click: 'Your name at the bottom left, then Profile'
				}
			],
			note: 'Do not see Users, Devices, or SMS Alerts in the sidebar? That is correct. Those are administrator pages.'
		},
		{
			title: 'Read the dashboard in ten seconds',
			summary: 'It answers "is anything happening, and is anything wrong" without any clicking.',
			steps: [
				{
					action: 'Look at the summary figures across the top.',
					detail: 'Totals for people, readers, openings, and records over the recent period.'
				},
				{
					action: 'Look at Live activity for the last ten things that happened.',
					detail:
						'It updates on its own as scans come in, so you can leave it open on a spare screen.'
				},
				{
					action: 'Follow anything that looks odd through to the audit log.',
					detail: 'Select the row, or open the audit log and search for the person.'
				}
			]
		},
		{
			title: 'Search the audit log properly',
			summary:
				'Every filter lives in the address bar, so a view you build can be pasted into a message and it will open the same way for somebody else.',
			steps: [
				{
					action: 'Narrow by outcome first.',
					detail: 'Granted, denied, or informational.',
					click: 'The Granted, Denied, or Info tab above the table'
				},
				{
					action: 'Narrow by what kind of event it was.',
					click: 'The Every event dropdown, then the event you want'
				},
				{
					action: 'Search for a person, a reader, a finger, or a message.',
					detail:
						'The search runs in the database over the whole trail, not just the rows on screen, so page 1 of the results really is the best match.',
					click: 'The search box above the table'
				},
				{
					action: 'Move through the results with the numbered pager under the table.',
					detail: 'The page number is in the address too, so a link points at the right page.'
				},
				{
					action: 'Select any row to see the full record.',
					detail:
						'The panel spells the event out in a sentence, including the reader, the finger, and the session it belonged to.'
				},
				{
					action: 'Copy the address once the view says what you mean.',
					detail:
						'For example a link ending in ?decision=denied&q=juan shows the refusals for one person and nothing else.'
				}
			],
			note: 'A person enrolled after an event still gets named on it. The log says "identified later, once that finger was enrolled" rather than rewriting history.'
		},
		{
			title: 'Work the flags page',
			summary:
				'This is the page to open at the start of a shift. It asks the awkward questions on its own.',
			steps: [
				{
					action: 'Set the window you care about.',
					click: 'The Last 14 days dropdown at the top right'
				},
				{
					action: 'Start with Needs attention.',
					detail: 'The count next to each tab tells you what is waiting before you click it.',
					click: 'The Needs attention tab'
				},
				{
					action: 'Open a group to see every occurrence with the time, the person, and the reader.',
					click: 'The row for the group'
				},
				{
					action: 'Read the paragraph under the group heading.',
					detail: 'It says what the check found and what to do about it.'
				},
				{
					action: 'Jump to the history for that person when you want the surrounding context.',
					click: 'See their trail, on the occurrence'
				}
			],
			note: 'Forgetfulness causes far more flags than dishonesty. Ask first, and ask kindly. A flag disappears as soon as the underlying record is put right.'
		},
		{
			title: 'Understand each check',
			summary: 'What each one means in practice, and the first question to ask.',
			steps: [
				{
					action: 'Opened the cabinet, recorded nothing.',
					detail:
						'The most important one. The cabinet issued a code and nobody spent it. Ask what they took, and have them record it late rather than not at all.'
				},
				{
					action: 'Cabinet tried without a door scan.',
					detail:
						'Somebody presented a finger at the cabinet with no door scan behind it. Usually they came in behind a classmate. This is the tailgating case, and the cabinet already refused it.'
				},
				{
					action: 'Checkout codes guessed.',
					detail:
						'Somebody typed codes the cabinet never gave them, more than once. One typo is nothing; a run of them is somebody trying to fabricate a record.'
				},
				{
					action: 'Unregistered fingers tried repeatedly.',
					detail:
						'A reader kept refusing unknown fingers. Either a class is enrolling, or somebody is trying fingers to see what opens.'
				},
				{
					action: 'Cabinet opened outside lab hours.',
					detail: 'Between 8 PM and 6 AM. Legitimate sometimes, but nobody is around to notice.'
				},
				{
					action: 'Opened repeatedly in one visit.',
					detail:
						'Several openings inside one door window. Often just forgetting an item, but it can also be materials leaving in more than one trip.'
				},
				{
					action: 'Disabled account still trying.',
					detail: 'A finger belonging to a disabled account is still being presented at a reader.'
				},
				{
					action: 'Failed sign-ins on the website.',
					detail:
						'Repeated wrong passwords for one account name. Could be a forgotten password, could be someone guessing.'
				},
				{
					action: 'Live window on a disabled account.',
					detail:
						'Somebody was disabled while their door window was still open. Worth closing the loop.'
				},
				{
					action: 'Reader has gone quiet.',
					detail:
						'A reader stopped reporting in. Tell the administrator: a reader that is offline is a reader that is recording nothing.'
				},
				{
					action: 'Enrollment codes never claimed.',
					detail:
						'Fingers were enrolled at a reader and nobody ever bound them to an account. Those fingers open nothing, and the person probably thinks they are enrolled.'
				}
			]
		},
		{
			title: 'Check what actually left the shelf',
			summary: 'The checkout list is your inventory record.',
			steps: [
				{
					action: 'Open the checkout page.',
					detail: 'As faculty you see the records for everybody here, not only your own.',
					click: 'Checkout, in the sidebar under Access'
				},
				{
					action: 'Search by person or by item to find a specific record.',
					detail: 'The search runs on the server across every record, not just the page in view.'
				},
				{
					action: 'Open a record to see the items, the quantities, and any photo.'
				},
				{
					action: 'Compare it with the reader events for the same minute when you need certainty.',
					detail:
						'Every record carries the opening it belongs to, because it could only be written with the code from that opening.'
				}
			]
		},
		{
			title: 'Help a student who cannot get in',
			summary: 'Almost every case is one of four things, in this order.',
			steps: [
				{
					action: 'Check they are enrolled at all.',
					detail: 'Search their name in the audit log. No enrollment event means no enrollment.'
				},
				{
					action: 'Check their account is still active.',
					detail:
						'A refusal reading "account disabled" is exactly that. Ask the administrator to re-enable them.'
				},
				{
					action: 'Check the reader is online.',
					detail:
						'The flags page raises "Reader has gone quiet" for a reader that stopped reporting in.'
				},
				{
					action: 'Watch them do the sequence once.',
					detail:
						'Most cabinet failures are somebody scanning the cabinet several minutes after the door. Have them scan the door and go straight to the cabinet.'
				},
				{
					action: 'Walk them through claiming their enrollment code if they never finished it.',
					detail:
						'The finger stays unclaimed, and useless, until the code is typed into the enrollment page.'
				}
			]
		},
		{
			title: 'What to hand to an administrator',
			summary: 'Four things are outside your reach by design.',
			steps: [
				{ action: 'Creating, editing, disabling, or deleting accounts.' },
				{ action: 'Resetting a password for somebody else.' },
				{ action: 'Pairing a reader, or replacing a reader that has failed.' },
				{
					action: 'Changing who receives text alerts.',
					detail:
						'You may already be on that list yourself: alerts go to administrators and faculty only.'
				}
			],
			note: 'When you report a problem, paste the audit log link for the exact view you were looking at. It saves the administrator from having to find it again.'
		},
		{
			title: 'A workable routine',
			summary: 'Ten minutes a week keeps the record trustworthy.',
			steps: [
				{
					action:
						'At the start of a shift, open the flags page and clear anything under Needs attention.',
					click: 'Flags, then the Needs attention tab'
				},
				{
					action: 'Once a week, set the window to 30 days and read the quieter findings too.',
					detail: 'Patterns show up over a month that never show up in a day.'
				},
				{
					action: 'After a busy class, skim the checkout list against the openings.',
					detail: 'This is when unrecorded openings happen, and it is easiest to fix the same day.'
				},
				{
					action: 'Report offline readers as soon as they appear.',
					detail: 'Everything else in the system depends on the readers reporting in.'
				}
			]
		}
	]
};
