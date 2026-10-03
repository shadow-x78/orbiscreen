
package com.orbiscreen.android.player

import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test

class StreamUrlRedactionTest {
    private val secret = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"

    @Test
    fun token_value_is_replaced() {
        val shown = redactQuery("token=$secret&session=sess-1&key=k1")
        assertFalse("token leaked: $shown", shown.contains(secret))
        assertEquals("token=REDACTED&session=sess-1&key=k1", shown)
    }

    @Test
    fun non_secret_parameters_survive() {
        val shown = redactQuery("token=$secret&session=sess-1&key=k1")
        assertTrue("session was dropped: $shown", shown.contains("session=sess-1"))
        assertTrue("key was dropped: $shown", shown.contains("key=k1"))
    }

    @Test
    fun a_query_without_a_token_is_unchanged() {
        assertEquals("session=sess-1&key=k1", redactQuery("session=sess-1&key=k1"))
    }

    @Test
    fun redaction_is_case_insensitive_on_the_parameter_name() {
        val shown = redactQuery("Token=$secret&session=s1")
        assertFalse("token leaked: $shown", shown.contains(secret))
        assertTrue(shown.contains("session=s1"))
    }

    @Test
    fun a_valueless_token_parameter_is_handled() {
        val shown = redactQuery("token&session=s1")
        assertEquals("token=REDACTED&session=s1", shown)
    }

    @Test
    fun an_empty_query_stays_empty() {
        assertEquals("", redactQuery(""))
    }
}
