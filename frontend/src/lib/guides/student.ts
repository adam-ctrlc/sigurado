import type { Guide } from '$lib/guides/types';

export const studentGuide: Guide = {
	slug: 'students',
	label: 'For students',
	title: 'Using Sigurado as a student',
	audience: 'Anyone who borrows materials from the stockroom.',
	intro:
		'Sigurado asks for your fingerprint twice: once at the door, once at the cabinet. That is the whole idea, and everything below follows from it. Read it once and you will never need to ask how the room works again.',
	sections: [
		{
			title: 'Understand the two scans',
			summary:
				'The cabinet only opens for the person who just walked through the door. Knowing why saves you a wasted trip.',
			steps: [
				{
					action: 'Scan at the door reader to get in.',
					detail:
						'A successful scan unlocks the door and quietly starts a short window, usually two minutes, that belongs to you alone.'
				},
				{
					action: 'Scan again at the cabinet reader while that window is still open.',
					detail:
						'The cabinet checks that the finger it just read is the same person the door let in. If it is, the lock releases.'
				},
				{
					action: 'If the window has closed, walk back and scan the door again.',
					detail:
						'Nothing is broken. The window is deliberately short so that somebody who walked in behind you cannot open the cabinet minutes later.'
				}
			],
			note: 'Never open the cabinet for somebody else. The record will say you took the materials, because as far as the system is concerned, you did.'
		},
		{
			title: 'Get your account',
			summary: 'Accounts are created by the lab administrator. You cannot sign yourself up.',
			steps: [
				{
					action: 'Ask the lab administrator or your instructor to create your account.',
					detail:
						'Give them your full name, your school email, and your mobile number if you have one.'
				},
				{
					action: 'Write down the username and temporary password they give you.',
					detail: 'Your username is usually built from your name, for example juan.r.delacruz.'
				}
			],
			note: 'Forgot your password? There is no self-service reset. Ask the administrator to set a new one for you.'
		},
		{
			title: 'Sign in for the first time',
			summary: 'Everything you can see on the website lives behind this one screen.',
			steps: [
				{
					action: 'Open the website and go to the sign-in page.',
					click: 'Sign in, on the top right of the landing page'
				},
				{
					action: 'Type your username or your email in the first field.',
					detail: 'Either one works. The field accepts both.'
				},
				{
					action: 'Type your password, then check it if you are unsure.',
					click: 'The eye icon at the right of the password field'
				},
				{ action: 'Submit the form.', click: 'Sign in' },
				{
					action: 'Change your temporary password straight away.',
					detail:
						'A password needs 8 to 16 characters with at least one capital letter, one small letter, and one symbol. The rules tick off as you type.',
					click: 'Your name at the bottom left, then Profile, then the Password card'
				}
			],
			note: 'If the page says the username or password is not right, check for a stray space and for Caps Lock before asking for a reset.'
		},
		{
			title: 'Fill in your profile',
			summary: 'The audit trail names you, so it is worth being named correctly.',
			steps: [
				{
					action: 'Open your profile.',
					click: 'Your name at the bottom left, then Profile'
				},
				{
					action: 'Fill in your first name, middle name, and last name.',
					detail:
						'These appear on every event you generate, so spell them the way you want them read.'
				},
				{
					action: 'Add your mobile number in the form 09xxxxxxxxx.',
					detail: 'Only staff receive text alerts, but a number on file lets someone reach you.'
				},
				{
					action: 'Let the site build your username for you if you would rather not invent one.',
					detail:
						'The button stays disabled until your first and last name are filled in, because it builds the username from them.',
					click: 'Generate, next to the Username field'
				},
				{ action: 'Save the card.', click: 'Save changes' }
			]
		},
		{
			title: 'Enroll your fingerprint',
			summary:
				'A finger has to be bound to your account before either reader will recognize it. This happens once, at the reader, then is confirmed on the website.',
			steps: [
				{
					action: 'Go to the reader and start an enrollment.',
					detail:
						'The reader asks for the same finger two or three times. Press flat, cover the sensor, and hold still until it beeps.'
				},
				{
					action: 'Read the one-time code the reader shows on its small screen.',
					detail: 'It is six characters, and it expires after a few minutes.'
				},
				{
					action: 'Open the enrollment page on the website.',
					click: 'Enrollment, in the sidebar under Access'
				},
				{
					action: 'Type the code into the boxes and confirm.',
					detail:
						'Until you do this, the finger belongs to nobody. This step is what ties it to your name.',
					click: 'Claim this fingerprint'
				},
				{
					action: 'Repeat at the other reader if your lab enrolls both separately.',
					detail: 'Ask the administrator which readers you need to be enrolled on.'
				}
			],
			note: 'Enroll a second finger if you can. A cut or a bandage on your index finger should not lock you out of the room.'
		},
		{
			title: 'Take materials the right way',
			summary:
				'Three actions in order: door, cabinet, record. The third one is the one people forget.',
			steps: [
				{ action: 'Scan at the door and go in.' },
				{
					action: 'Scan at the cabinet while your window is still open.',
					detail: 'The cabinet unlocks and shows a six character code on its screen.'
				},
				{
					action: 'Write that code down or keep it on screen.',
					detail:
						'You need it to record what you took, and it is only good for fifteen minutes. It is never shown anywhere else, on purpose.'
				},
				{ action: 'Take what you need and close the cabinet firmly.' },
				{
					action: 'Record the checkout before you leave.',
					click: 'Checkout, in the sidebar under Access'
				}
			],
			note: 'The cabinet sends a text message to the lab staff every time it opens. That is normal and is not a sign you did anything wrong.'
		},
		{
			title: 'Record what you took',
			summary:
				'The record is the whole point of the system. An opening with no record behind it is flagged for staff to chase, and your name is on it.',
			steps: [
				{
					action: 'Open the checkout page.',
					click: 'Checkout, in the sidebar under Access'
				},
				{
					action: 'Type the code from the cabinet into the six boxes.',
					detail:
						'If the page says the cabinet has not issued a code for you, the cabinet has not been opened by you recently, or your fifteen minutes ran out. Scan again for a new code.'
				},
				{
					action: 'Fill in the item table: what you took on the left, how many on the right.',
					detail: 'One row per item. Add rows if you need more than three.',
					click: 'Add item'
				},
				{
					action:
						'Attach a photo if you want one, which is optional but settles arguments quickly.',
					click: 'Choose photo'
				},
				{ action: 'Submit the record.', click: 'Record checkout' },
				{
					action: 'Check it appears in the list below the form.',
					detail: 'If it is there, you are finished and the code is spent.'
				}
			],
			note: 'Returning materials? Tell the staff. Sigurado records what left the shelf; the return is a conversation, not a form.'
		},
		{
			title: 'Keep an eye on your own record',
			summary:
				'You can see everything the system holds about you, and you should look now and then.',
			steps: [
				{
					action: 'Open your dashboard for a summary.',
					detail:
						'Your recent openings, your checkouts, and whether a fingerprint is bound to you.',
					click: 'Dashboard, in the sidebar under Monitoring'
				},
				{
					action: 'Review your checkout history at the bottom of the checkout page.',
					detail: 'Students see their own records only.'
				},
				{
					action: 'Review your recent sign-ins on your profile.',
					detail:
						'Each row shows the time, the browser, and the address. A sign-in you do not recognize means your password needs changing today.',
					click: 'Profile, then the Recent sign-ins card'
				}
			]
		},
		{
			title: 'When something does not work',
			summary: 'Most problems are one of five things. Work down the list before asking for help.',
			steps: [
				{
					action:
						'The door will not open: your finger may not be enrolled, or your account may be disabled.',
					detail: 'Try the other enrolled finger, then ask the administrator to check your account.'
				},
				{
					action: 'The cabinet will not open: your door window has probably closed.',
					detail: 'Go back to the door, scan again, and return to the cabinet promptly.'
				},
				{
					action: 'The reader does not react at all: it may be offline.',
					detail: 'Tell the staff. The Devices page tells them which reader has gone quiet.'
				},
				{
					action: 'The code will not work: it expires after fifteen minutes and works only once.',
					detail:
						'Scan at the cabinet again for a fresh one, and record the checkout straight away.'
				},
				{
					action: 'You forgot to record a checkout: say so now rather than waiting.',
					detail:
						'Staff already see a flag for it. Telling them turns a suspicious gap into a corrected record.'
				}
			]
		}
	]
};
