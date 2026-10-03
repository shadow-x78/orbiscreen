package com.orbiscreen.android.input

import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.async
import kotlinx.coroutines.delay
import kotlinx.coroutines.runBlocking
import kotlinx.coroutines.withTimeout
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test
import java.util.concurrent.CountDownLatch
import java.util.concurrent.TimeUnit

class InputDispatcherTest {
    @Test
    fun resizedCursorUsesNewBoundsForMovementAndPlacement() {
        val cursor = VirtualCursor(100, 80)
        cursor.resize(200, 160)
        cursor.applyDelta(500f, 500f)
        assertEquals(199f, cursor.x, 0f)
        assertEquals(159f, cursor.y, 0f)
        cursor.resize(20, 10)
        assertEquals(19f, cursor.x, 0f)
        assertEquals(9f, cursor.y, 0f)
        cursor.applyDelta(500f, 500f)
        assertEquals(19f, cursor.x, 0f)
        assertEquals(9f, cursor.y, 0f)
        cursor.place(50f, -50f)
        assertEquals(19f, cursor.x, 0f)
        assertEquals(0f, cursor.y, 0f)
    }

    @Test
    fun cursorRetainsFractionalMovementAndConfinesEachDelta() {
        val cursor = VirtualCursor(100, 80)
        cursor.applyDelta(0.1f, 0.1f)
        assertEquals(50.1f, cursor.x, 0.0001f)
        assertEquals(40.1f, cursor.y, 0.0001f)
        cursor.applyDelta(500f, -500f)
        cursor.applyDelta(-1f, 1f)
        assertEquals(98f, cursor.x, 0f)
        assertEquals(1f, cursor.y, 0f)
    }

    @Test
    fun degenerateCursorBoundsStayNonNegative() {
        val cursor = VirtualCursor(0, 0)
        assertEquals(0f, cursor.x, 0f)
        assertEquals(0f, cursor.y, 0f)
        cursor.resize(1, 1)
        cursor.applyDelta(50f, -50f)
        assertEquals(0f, cursor.x, 0f)
        assertEquals(0f, cursor.y, 0f)
    }

    @Test
    fun contiguousMotionCoalescesWithoutCrossingClickBatch() = runBlocking {
        val queue = OrderedInputQueue<String>()
        queue.submit(listOf("move1"), "pointer")
        queue.submit(listOf("move2"), "pointer")
        queue.submit(listOf("click-position", "down", "up"))
        queue.submit(listOf("move3"), "pointer")
        queue.submit(listOf("move4"), "pointer")
        queue.close()
        val delivered = mutableListOf<String>()
        queue.drain { v, _ -> delivered.add(v) }
        assertEquals(listOf("move2", "click-position", "down", "up", "move4"), delivered)
    }

    @Test
    fun motionBeforeStylusAndTouchReleasesIsNotSentAfterRelease() = runBlocking {
        val queue = OrderedInputQueue<String>()
        queue.submit(listOf("stylus-down"))
        queue.submit(listOf("stylus1"), "stylus")
        queue.submit(listOf("stylus2"), "stylus")
        queue.submit(listOf("stylus-up"))
        queue.submit(listOf("touch-down"))
        queue.submit(listOf("touch1"), "touch:0:1")
        queue.submit(listOf("touch2"), "touch:0:1")
        queue.submit(listOf("touch-up"))
        queue.close()
        val delivered = mutableListOf<String>()
        queue.drain { v, _ -> delivered.add(v) }
        assertEquals(
            listOf("stylus-down", "stylus2", "stylus-up", "touch-down", "touch2", "touch-up"),
            delivered,
        )
    }

    @Test
    fun differentMotionSourcesPreserveTheirOrder() = runBlocking {
        val queue = OrderedInputQueue<String>()
        queue.submit(listOf("pointer1"), "pointer")
        queue.submit(listOf("touch0"), "touch:0:1")
        queue.submit(listOf("touch1"), "touch:1:2")
        queue.submit(listOf("pointer2"), "pointer")
        queue.submit(listOf("key-up"))
        queue.close()
        val delivered = mutableListOf<String>()
        queue.drain { v, _ -> delivered.add(v) }
        assertEquals(listOf("pointer1", "touch0", "touch1", "pointer2", "key-up"), delivered)
    }

    @Test
    fun aFullQueueNeverBlocksTheProducerAndStillDeliversEveryDiscrete() = runBlocking {
        val queue = OrderedInputQueue<Int>(capacity = 2)
        queue.submit(listOf(0))
        queue.submit(listOf(1))
        val started = CountDownLatch(1)
        val submitted = CountDownLatch(1)
        val producer = async(Dispatchers.IO) {
            started.countDown()
            repeat(128) { assertTrue(queue.submit(listOf(it + 2))) }
            submitted.countDown()
            queue.close()
        }
        assertTrue(started.await(2, TimeUnit.SECONDS))
        assertTrue(
            "producer blocked on a full queue",
            submitted.await(2, TimeUnit.SECONDS)
        )
        val delivered = mutableListOf<Int>()
        withTimeout(5_000) {
            queue.drain { v, _ -> delivered.add(v) }
            producer.await()
        }
        assertEquals((0..129).toList(), delivered)
    }

    @Test
    fun motionIsDroppedRatherThanParkedWhenTheQueueIsFull() = runBlocking {
        val queue = OrderedInputQueue<String>(capacity = 2)
        queue.submit(listOf("a"))
        queue.submit(listOf("b"))
        assertFalse(queue.submit(listOf("c"), motionKey = "pointer"))
        queue.close()
        val delivered = mutableListOf<String>()
        withTimeout(5_000) { queue.drain { v, _ -> delivered.add(v) } }
        assertEquals(listOf("a", "b"), delivered)
    }

    @Test
    fun aDroppedMotionSampleDoesNotPoisonLaterMotion() = runBlocking {
        val queue = OrderedInputQueue<String>(capacity = 2)
        queue.submit(listOf("a"))
        queue.submit(listOf("b"))
        assertFalse(queue.submit(listOf("dropped"), motionKey = "pointer"))
        val delivered = mutableListOf<String>()
        val consumer = async(Dispatchers.IO) {
            queue.drain { v, _ -> delivered.add(v) }
        }
        withTimeout(5_000) {
            repeat(8) {
                delay(30)
                queue.submit(listOf("move$it"), motionKey = "pointer")
            }
            queue.close()
            consumer.await()
        }
        assertTrue(
            "later motion was discarded after a drop: $delivered",
            delivered.any { it.startsWith("move") }
        )
    }

    @Test
    fun inFlightDeliveryCannotBeOverwrittenByCoalescing() = runBlocking {
        val queue = OrderedInputQueue<String>()
        queue.submit(listOf("first"), "pointer")
        val entered = CountDownLatch(1)
        val finish = CountDownLatch(1)
        val delivered = mutableListOf<String>()
        val consumer = async(Dispatchers.IO) {
            queue.drain { v, _ ->
                if (v == "first") {
                    entered.countDown()
                    assertTrue(finish.await(5, TimeUnit.SECONDS))
                }
                delivered.add(v)
            }
        }
        try {
            assertTrue(entered.await(2, TimeUnit.SECONDS))
            queue.submit(listOf("second"), "pointer")
            queue.submit(listOf("third"), "pointer")
            queue.submit(listOf("release"))
            queue.close()
        } finally {
            finish.countDown()
        }
        withTimeout(5_000) { consumer.await() }
        assertEquals(listOf("first", "third", "release"), delivered)
    }

    @Test
    fun closeDuringBackpressureStillDrainsQueuedRelease() = runBlocking {
        val queue = OrderedInputQueue<String>(capacity = 1)
        queue.submit(listOf("down"))
        val started = CountDownLatch(1)
        val producer = async(Dispatchers.IO) {
            started.countDown()
            queue.submit(listOf("up"))
        }
        assertTrue(started.await(2, TimeUnit.SECONDS))
        queue.close()
        val delivered = mutableListOf<String>()
        withTimeout(5_000) {
            queue.drain { v, _ -> delivered.add(v) }
            val accepted = producer.await()
            assertEquals(if (accepted) listOf("down", "up") else listOf("down"), delivered)
        }
    }

    @Test
    fun closeDrainsAcceptedInputAndRejectsNewInput() = runBlocking {
        val queue = OrderedInputQueue<String>()
        queue.submit(listOf("down", "up"))
        queue.close()
        queue.close()
        assertFalse(queue.submit(listOf("late")))
        val delivered = mutableListOf<String>()
        queue.drain { v, _ -> delivered.add(v) }
        assertEquals(listOf("down", "up"), delivered)
    }

    @Test
    fun theBacklogIsCappedRatherThanGrowingWithoutBound() {
        val queue = OrderedInputQueue<Int>(capacity = 1)
        queue.submit(listOf(0))
        val accepted = (1..OrderedInputQueue.OVERFLOW_CAPACITY + 50).count { queue.submit(listOf(it)) }
        assertTrue(
            "backlog accepted everything: accepted=$accepted dropped=${queue.droppedDiscretes}",
            queue.droppedDiscretes > 0
        )
        queue.close()
    }
}
