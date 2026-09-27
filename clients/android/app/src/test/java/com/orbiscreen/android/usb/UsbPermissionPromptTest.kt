// Orbiscreen - UsbPermissionPromptTest.kt (GPL-3.0-or-later)
// https://github.com/shadow-x78/orbiscreen

package com.orbiscreen.android.usb

import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test

class UsbPermissionPromptTest {

    @Test
    fun idleAllowsOnePrompt() {
        val prompt = UsbPermissionPrompt()
        assertTrue(prompt.tryBeginPrompt())
        assertEquals(UsbPermissionPrompt.State.Prompting, prompt.state)
        assertFalse(prompt.tryBeginPrompt())
    }

    @Test
    fun cancelThenResumeDoesNotReprompt() {
        val prompt = UsbPermissionPrompt()
        assertTrue(prompt.tryBeginPrompt())
        // onResume can race the denied broadcast; Prompting must still block.
        assertFalse(prompt.tryBeginPrompt())
        prompt.onDenied()
        assertEquals(UsbPermissionPrompt.State.Denied, prompt.state)
        assertFalse(prompt.tryBeginPrompt())
    }

    @Test
    fun forceRetriesWhilePromptingOrDenied() {
        val prompt = UsbPermissionPrompt()
        assertTrue(prompt.tryBeginPrompt())
        assertTrue(prompt.tryBeginPrompt(force = true))
        prompt.onDenied()
        assertTrue(prompt.tryBeginPrompt(force = true))
        assertEquals(UsbPermissionPrompt.State.Prompting, prompt.state)
    }

    @Test
    fun explicitRetryAfterDenyPromptsAgain() {
        val prompt = UsbPermissionPrompt()
        assertTrue(prompt.tryBeginPrompt())
        prompt.onDenied()
        assertTrue(prompt.tryBeginPrompt(force = true))
        assertEquals(UsbPermissionPrompt.State.Prompting, prompt.state)
    }

    @Test
    fun grantThenLaterInitMayPromptAgain() {
        val prompt = UsbPermissionPrompt()
        assertTrue(prompt.tryBeginPrompt())
        prompt.onGranted()
        assertEquals(UsbPermissionPrompt.State.Idle, prompt.state)
        assertTrue(prompt.tryBeginPrompt())
    }

    @Test
    fun detachClearsDeniedSoReattachPrompts() {
        val prompt = UsbPermissionPrompt()
        assertTrue(prompt.tryBeginPrompt())
        prompt.onDenied()
        prompt.onDetached()
        assertEquals(UsbPermissionPrompt.State.Idle, prompt.state)
        assertTrue(prompt.tryBeginPrompt())
    }

    @Test
    fun failedRequestReturnsToIdle() {
        val prompt = UsbPermissionPrompt()
        assertTrue(prompt.tryBeginPrompt())
        prompt.onPromptFailed()
        assertEquals(UsbPermissionPrompt.State.Idle, prompt.state)
        assertTrue(prompt.tryBeginPrompt())
    }

    @Test
    fun failedRequestDoesNotClearDenied() {
        val prompt = UsbPermissionPrompt()
        assertTrue(prompt.tryBeginPrompt())
        prompt.onDenied()
        prompt.onPromptFailed()
        assertEquals(UsbPermissionPrompt.State.Denied, prompt.state)
        assertFalse(prompt.tryBeginPrompt())
    }
}
