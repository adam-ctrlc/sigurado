import type { Guide } from '$lib/guides/types';

export const adminGuide: Guide = {
	slug: 'admin',
	label: 'For administrators',
	title: 'Running Sigurado as an administrator',
	audience: 'Whoever owns the stockroom: accounts, readers, alerts, and the record itself.',
	intro:
		'You can do everything faculty can do, plus the four things nobody else can: create accounts, pair readers, choose who gets texted, and delete things. Read the faculty guide as well, because the audit log and flags pages are your daily work too. This one covers what only you can reach.',
	sections: [
		{
			title: 'Set the system up the first time',
			summary: 'The order matters. A reader with no accounts behind it cannot enroll anyone.',
			steps: [
				{
					action: 'Sign in with the administrator account created when the database was seeded.',
					detail: 'Change its password on your profile before anything else happens.'
				},
				{
					action: 'Create the accounts for your staff and students.',
					click: 'Users, in the sidebar under Administration'
				},
				{
					action: 'Pair the door node and the cabinet node.',
					click: 'Devices, in the sidebar under Administration, then the Inventory tab'
				},
				{
					action: 'Flash each node with its own device secret.',
					detail: 'The secret is shown once, at pairing. Copy it into the firmware then and there.'
				},
				{
					action: 'Choose who receives text alerts.',
					click: 'SMS Alerts, then the Recipients tab'
				},
				{
					action: 'Enroll fingerprints, starting with your own.',
					detail: 'Enroll at the reader, then claim the code on the enrollment page.'
				},
				{
					action: 'Do one full run yourself: door, cabinet, record.',
					detail:
						'It proves the whole chain end to end, and it puts a known-good example in the audit log.'
				}
			]
		},
		{
			title: 'Create an account',
			summary: 'Nobody can sign themselves up. Every account starts here.',
			steps: [
				{
					action: 'Open the roster.',
					click: 'Users, in the sidebar under Administration'
				},
				{ action: 'Start a new account.', click: 'Add user, at the top right' },
				{
					action: 'Fill in the name in parts: first, middle, last, and a suffix if they use one.',
					detail:
						'These are separate fields on purpose. The audit log builds the displayed name from them, so "Dela Cruz" stays a surname and does not become a middle name.'
				},
				{
					action: 'Set a username, or let the form build one from the name.',
					detail: 'The pattern is first.middleinitial.last, for example juan.r.delacruz.'
				},
				{
					action: 'Add the school email and the mobile number in the form 09171234567.',
					detail:
						'The number matters for staff, because text alerts are sent to the number on the account.'
				},
				{
					action: 'Pick the role.',
					detail:
						'Student: their own records only. Faculty: sees everything, changes nothing. Administrator: everything, including this page.'
				},
				{
					action: 'Set a first password that meets the policy.',
					detail:
						'8 to 16 characters, at least one capital, one small letter, and one symbol, no spaces. The rules tick off as you type.'
				},
				{ action: 'Save.', click: 'Save changes' },
				{
					action: 'Hand over the username and password, and ask them to change it at first sign-in.'
				}
			],
			note: 'Give people the lowest role that lets them work. Faculty is enough for supervising a class; administrator is for whoever owns the room.'
		},
		{
			title: 'Maintain the roster',
			summary: 'Search, filter, and paging all run on the server, so the roster stays quick.',
			steps: [
				{
					action: 'Find somebody by name, username, or email.',
					click: 'The search box above the table'
				},
				{
					action: 'Narrow to one role, or to active and disabled accounts.',
					click: 'The dropdowns beside the search box'
				},
				{
					action: 'Open the actions for a person.',
					click: 'The three dots at the end of their row'
				},
				{
					action: 'Correct a name, username, email, number, or role.',
					click: 'Edit details'
				},
				{
					action: 'Review where and when they have signed in.',
					detail: 'Time, browser, and address for each sign-in, a page at a time.',
					click: 'Sign-in history'
				},
				{
					action: 'Disable somebody who should no longer get in.',
					detail:
						'This is the right move when a student leaves. The readers refuse them immediately, and every event they ever generated stays in the trail.',
					click: 'Disable account'
				},
				{
					action: 'Re-enable them when they come back.',
					click: 'Enable account'
				},
				{
					action: 'Delete only when an account was created in error.',
					detail: 'Deleting removes the person. Disabling is almost always the better answer.',
					click: 'Delete user, then confirm'
				}
			],
			note: 'Your own row has no Disable and no Delete, and you cannot change your own role. That is deliberate: it stops you locking yourself out of the only administrator account. Another administrator can still do all three to you.'
		},
		{
			title: 'Handle passwords',
			summary: 'There is no email reset, so resets come through you.',
			steps: [
				{
					action: 'Set a new password for somebody who is locked out.',
					click: 'Users, the three dots on their row, Edit details, then the password field'
				},
				{
					action: 'Tell them to change it once they are in.',
					click: 'Profile, then the Password card'
				},
				{
					action: 'Check the flags page before resetting a password twice in a week.',
					detail:
						'"Failed sign-ins on the website" tells you whether it is a forgotten password or somebody guessing.'
				},
				{
					action: 'Change your own password from your profile like everyone else.'
				}
			]
		},
		{
			title: 'Pair and look after the readers',
			summary:
				'A reader that is not reporting in is recording nothing, so this page matters daily.',
			steps: [
				{
					action: 'Watch which readers are live.',
					detail:
						'A reader that has not reported in for ninety seconds is treated as offline, with the time it was last heard from.',
					click: 'Devices, then the Status tab'
				},
				{
					action: 'Pair a new node.',
					detail:
						'Name it for where it is, for example door-1 or cabinet-1, and say whether it carries the GSM module.',
					click: 'Devices, the Inventory tab, then Add device'
				},
				{
					action: 'Copy the secret it shows you, once, and put it in the firmware.',
					detail:
						'The node sends its id and secret on every request; without them nothing is accepted.'
				},
				{
					action: 'Rotate a secret you think has leaked.',
					detail:
						'The old secret stops working the moment the new one is issued, so reflash promptly.',
					click: 'The three dots on the device row, then Rotate secret'
				},
				{
					action: 'Mark which node carries the SIM800L.',
					detail: 'Only a node with the modem drains the text message queue.',
					click: 'The three dots on the device row, then the SMS option'
				},
				{
					action: 'Review the connection history of a reader that has been unreliable.',
					detail:
						'The periods it was up and down tell you whether it is power, wiring, or the network.'
				}
			]
		},
		{
			title: 'Set up text alerts',
			summary:
				'The cabinet node has a SIM800L on it. The server queues messages and the node sends them, so alerts still arrive when nobody is watching a screen.',
			steps: [
				{
					action: 'Open the recipient list.',
					click: 'SMS Alerts, then the Recipients tab'
				},
				{
					action: 'Add somebody.',
					detail:
						'Recipients are chosen from the accounts you already have, so a number never has to be typed twice.',
					click: 'Add recipient'
				},
				{
					action: 'Pick the person from the searchable list.',
					detail:
						'Only administrators and faculty appear, because alerts are an oversight tool. Students cannot be added.',
					click: 'The Person field, then type to search'
				},
				{
					action: 'Add their number if the account has none.',
					detail: 'It is saved onto their profile, so the number lives in one place only.'
				},
				{
					action: 'Choose what they get texted about.',
					detail:
						'The cabinet opens: recommended, this is the one that matters. Access is refused: catches tailgating. The door opens: noisy, every entry sends a text.'
				},
				{ action: 'Save.', click: 'Add recipient' },
				{
					action: 'Prove it works before you rely on it.',
					click: 'The three dots on their row, then Send a test'
				},
				{
					action: 'Watch what actually went out.',
					detail:
						'Queued, sent, or failed, with the attempt count. A message is retried up to three times.',
					click: 'The Outbox tab'
				}
			],
			note: 'The last recipient cannot be removed, by the page and by the server both. A stockroom with alerts switched off entirely is not a state worth allowing by accident.'
		},
		{
			title: 'Read the trail as a spreadsheet',
			summary:
				'The Sheet page is the whole trail laid out in rows and columns, built from what the readers posted here. Nothing to connect, no account, no keys.',
			steps: [
				{
					action: 'Open it.',
					click: 'Sheet, in the sidebar under Administration'
				},
				{
					action: 'Pick a tab along the bottom, the way you would in a spreadsheet.',
					detail:
						'Events and Checkouts are the growing log. Flags and Daily are marked derived, because they describe the last 30 days rather than a moment, and they recalculate every time you look.'
				},
				{
					action: 'Select any cell to read it in full in the bar above the grid.',
					detail:
						'Columns are truncated to keep the grid readable, so that bar is how you read a long detail. Arrow keys move the selection.'
				},
				{
					action: 'Search across every column at once.',
					detail:
						'The search runs in the database over the whole tab, not just the rows on screen, and it stays with you when you switch tabs.',
					click: 'Search every column'
				},
				{
					action: 'Change how many rows you see, and page through the rest.',
					detail:
						'Row numbers are absolute, so row 27 really is the twenty-seventh row rather than the second row of page two.'
				},
				{
					action: 'Download the tab when somebody wants their own copy.',
					detail:
						'You get a CSV that opens in Excel, Google Sheets or Numbers, with whatever search you had applied.',
					click: 'CSV'
				}
			],
			note: 'This page and the audit log show the same facts in two shapes: the log is for reading one event closely, the sheet is for scanning and exporting many. Neither can be edited, here or anywhere.'
		},
		{
			title: 'Optional: also mirror it to Google Sheets',
			summary:
				'Only worth it if somebody needs the data inside their own Google account. The page above already works without it.',
			steps: [
				{
					action: 'Decide whether you actually need it.',
					detail:
						'The Sheet page and the CSV download cover most reasons people ask for a spreadsheet. The mirror adds a live copy in Google Drive, and a Google Cloud project to look after.'
				},
				{
					action: 'Make a spreadsheet and copy its id from the address bar.',
					detail: 'The long piece between /d/ and /edit.'
				},
				{
					action: 'Enable the Google Sheets API in a Google Cloud project, then create a service account with a JSON key.'
				},
				{
					action: 'Share the spreadsheet with the service account address as an Editor.',
					detail:
						'The address is the client_email inside the JSON file. Skipping this is the most common mistake, and it shows up as a 403 refusal.'
				},
				{
					action: 'Set SHEETS_ENABLED, SHEETS_SPREADSHEET_ID and SHEETS_KEY_PATH in the backend .env, then restart it.',
					detail:
						'A panel then appears under the sheet showing when it last pushed and how many rows are still to send.'
				},
				{
					action: 'Push it yourself when you do not want to wait for the timer.',
					click: 'Push now'
				}
			],
			note: 'The push runs on the server, never on a reader, so no Google key is ever stored on a board somebody could unscrew from a wall.'
		},
		{
			title: 'Triage the flags',
			summary:
				'Same page faculty use, but the fixes are usually yours. Read the faculty guide for what each check means.',
			steps: [
				{
					action: 'Clear Needs attention first.',
					click: 'Flags, then the Needs attention tab'
				},
				{
					action: 'Unrecorded openings: chase the person, and have the record written late.',
					detail: 'A late record beats a permanent gap, and the flag clears once it exists.'
				},
				{
					action:
						'Repeated code guessing: talk to the person, then consider disabling the account.',
					detail: 'This is somebody trying to write a record for materials they did not sign for.'
				},
				{
					action:
						'Repeated failed sign-ins: reset the password, or disable the account if it was not them.'
				},
				{
					action: 'A quiet reader: check power, wiring, and network, in that order.',
					click: 'Devices, then the Status tab'
				},
				{
					action: 'Unclaimed enrollments: tell the person to finish claiming their code.',
					detail: 'They almost certainly believe they are enrolled, and they are not.'
				},
				{
					action: 'Widen the window to 30 or 90 days once a month.',
					detail: 'Slow patterns are invisible in a fortnight.',
					click: 'The Last 14 days dropdown'
				}
			]
		},
		{
			title: 'Know what you cannot change',
			summary: 'Some things are fixed on purpose. Knowing which saves you looking for a setting.',
			steps: [
				{
					action: 'Reader events cannot be edited or deleted.',
					detail: 'The trail is only worth having because nobody can tidy it up, including you.'
				},
				{
					action: 'Checkout records cannot be rewritten.',
					detail: 'A wrong record is corrected by adding the right one and explaining it.'
				},
				{
					action: 'A checkout cannot be recorded without a code from a real opening.',
					detail: 'This is what stops materials being logged from a laptop at home.'
				},
				{
					action: 'You cannot disable, delete, or demote yourself.',
					detail: 'Another administrator can. Keep a second administrator account for exactly that.'
				},
				{
					action: 'The last text recipient cannot be removed.'
				}
			]
		},
		{
			title: 'Keep it healthy',
			summary: 'Short, boring habits. They are what makes the record trustworthy a year from now.',
			steps: [
				{
					action: 'Daily: glance at Devices, Status. Any reader offline is urgent.'
				},
				{
					action: 'Daily: clear Needs attention on the flags page.'
				},
				{
					action:
						'Weekly: skim the outbox for failed messages, which usually means signal or credit.'
				},
				{
					action: 'Each term: disable the accounts of people who have left.',
					detail: 'Disable, do not delete. Their history stays meaningful.'
				},
				{
					action: 'Each term: check the recipient list still matches who actually works here.'
				},
				{
					action: 'Keep a second administrator account, and make sure somebody else can reach it.',
					detail:
						'One administrator with one password is a single point of failure for the whole room.'
				},
				{
					action: 'Back up the database on whatever schedule your school already uses.',
					detail: 'The audit trail is the deliverable. Everything else can be rebuilt.'
				}
			]
		}
	]
};
