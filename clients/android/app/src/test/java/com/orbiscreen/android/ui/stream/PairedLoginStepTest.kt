// Orbiscreen - PairedLoginStepTest.kt (GPL-3.0-or-later)

package com.orbiscreen.android.ui.stream

import com.orbiscreen.android.net.HostApi
import org.junit.Assert.assertEquals
import org.junit.Test

class PairedLoginStepTest {
    private val pinned = "A".repeat(64)
    private val other = "B".repeat(64)

    @Test
    fun matchingValidCertificateConnectsWithTheSavedLogin() {
        assertEquals(PairedLoginStep.Connect, pairedLoginStep(pinned, HostApi.ObservedCertificate(pinned, validNow = true)))
    }

    @Test
    fun matchingButExpiredCertificateIsReportedNotRePaired() {
        assertEquals(PairedLoginStep.Expired, pairedLoginStep(pinned, HostApi.ObservedCertificate(pinned, validNow = false)))
    }

    @Test
    fun aDifferentCertificateAsksForReconfirmationInsteadOfFailing() {
        assertEquals(PairedLoginStep.Reconfirm, pairedLoginStep(pinned, HostApi.ObservedCertificate(other, validNow = true)))
    }

    @Test
    fun anExpiredCertificateWinsOverAChangedOne() {
        // The pinned client refuses expired certificates, so re-confirming a new but expired one would fail too.
        assertEquals(PairedLoginStep.Expired, pairedLoginStep(pinned, HostApi.ObservedCertificate(other, validNow = false)))
    }
}
